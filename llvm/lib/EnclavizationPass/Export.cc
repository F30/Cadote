#include <fstream>
#include <ios>
#include <sstream>
#include <string>

#include "llvm/IR/Argument.h"
#include "llvm/IR/Attributes.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/Type.h"
#include <llvm/Support/raw_ostream.h>

#include "EnclavizationPass/Export.h"
#include "EnclavizationPass/Names.h"


using namespace llvm;

// EDL type used in case a pointer type would not be valid (e.g. for return values)
#define EDL_PTR_REPLACEMENT "uint64_t"


EdlFile::EdlFile(std::string path) {
  fstream = std::ofstream(path, std::ios_base::trunc);
  fstream.exceptions(std::ofstream::badbit | std::ofstream::failbit);

  fstream << "enclave {\n";
  // Add Teaclave SGX SDK default EDL files
  fstream << "from \"sgx_tstd.edl\" import *;\n";
  fstream << "from \"sgx_stdio.edl\" import *;\n";
  fstream << "from \"sgx_backtrace.edl\" import *;\n";
  fstream << "from \"sgx_tstdc.edl\" import *;\n";

  fstream << "\n";
  fstream << "trusted {\n";
}

EdlFile::~EdlFile() {
  fstream << "};\n";
  fstream << "};\n";

  fstream.flush();
  fstream.close();
}

void EdlFile::addFunction(llvm::Function *postgateFunc) {
  fstream << "public ";
  if (postgateFunc->getReturnType()->isPointerTy()) {
    // In most cases, pointers returned from functions will point back to input values (i.e. outside the
    // enclave) because of lifetimes, so this is hacky but safe
    // TODO: Heap allocations, which are an exemption from that
    fstream << EDL_PTR_REPLACEMENT;
  } else {
    fstream << getEdlType(postgateFunc->getReturnType(), postgateFunc->getContext());
  }
  fstream << " " << postgateFunc->getName().str() << "(";

  for (size_t i = 0; i < postgateFunc->arg_size(); ++i) {
    if (i != 0) {
      fstream << ", ";
    }

    auto arg = postgateFunc->getArg(i);
    auto argType = arg->getType();

    if (argType->isPointerTy()) {
      auto edlAttrs = getEdlAttributes(arg);
      if (!edlAttrs.empty()) {
        fstream << "[";

        for (auto attr = edlAttrs.begin(); attr != edlAttrs.end(); ++attr) {
          if (attr != edlAttrs.begin()) {
            fstream << ", ";
          }
          fstream << *attr;
        }

        fstream << "] ";
      }
    }

    auto edlType = getEdlType(arg->getType(), postgateFunc->getContext());
    fstream << edlType << " arg" << i;
  }

  fstream << ");\n";
  fstream.flush();
}

std::string EdlFile::getEdlType(Type *llvmType, LLVMContext &context) {
  std::string edlType;
  raw_string_ostream typeStream(edlType);

  if (llvmType->isPointerTy()) {
    auto pointedToType = llvmType->getPointerElementType();
    // rustc will never give us struct or array types as arguments, only pointers to them
    if (pointedToType->isStructTy() || pointedToType->isArrayTy()) {
      // The actual size is contained in the `dereferencable` attribute
      typeStream << "int8_t *";
    } else {
      typeStream << getEdlType(pointedToType, context) << " *";
    }
  }

  else if (llvmType == Type::getVoidTy(context)) {
    typeStream << "void";
  } else if (llvmType == Type::getInt8Ty(context)) {
    typeStream << "int8_t";
  } else if (llvmType == Type::getInt16Ty(context)) {
    typeStream << "int16_t";
  } else if (llvmType == Type::getInt32Ty(context)) {
    typeStream << "int32_t";
  } else if (llvmType == Type::getInt64Ty(context)) {
    typeStream << "int64_t";
  } else if (llvmType == Type::getFloatTy(context)) {
    typeStream << "float";
  } else if (llvmType == Type::getDoubleTy(context)) {
    typeStream << "double";
  } else {
    typeStream << "UNKNOWN<";
    llvmType->print(typeStream);
    typeStream << "> ";
  }

  return edlType;
}

std::vector<std::string> EdlFile::getEdlAttributes(Argument *arg) {
    std::vector<std::string> attrs;

    if (arg->hasAttribute(Attribute::AttrKind::StructRet)) {
      attrs.push_back("out");
    }
    if (arg->hasAttribute(Attribute::AttrKind::ReadOnly)) {
      // LLVM's semantics only guarantee that "the function does not write through this pointer argument,
      // even though it may write to the memory that the pointer points to"
      // However, rustc (as of nightly-2020-10-25) will set this for (completely) unmutable references
      attrs.push_back("in");
    }

    if (arg->hasAttribute(Attribute::AttrKind::Dereferenceable)) {
      uint64_t numBytes = arg->getAttribute(Attribute::AttrKind::Dereferenceable).getValueAsInt();
      std::stringstream attrStream;
      attrStream << "size=" << numBytes;
      attrs.push_back(attrStream.str());
    } else {
      attrs.push_back("user_check");
    }

    return attrs;
}
