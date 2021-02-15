#include <stdexcept>
#include <string.h>

#include "llvm/Passes/PassPlugin.h"
#include "llvm/Passes/PassBuilder.h"
#include "llvm/Transforms/Utils/BasicBlockUtils.h"

#include "rustc_demangle.h"
#include "IndirectionPass.h"


using namespace llvm;

#define DEBUG_TYPE "indirection-pass"

#define FUNC_SUFFIX "_wrapped_"
#define DEMANGLED_LEN_MAX 200


bool IndirectionPass::runOnModule(Module &mod) {
  bool didWrap = false;

  for (auto &func : mod) {
    if (func.isDeclaration()) {
      continue;
    }

    // First generate list of calls to be wrapped, since modification of a BasicBlock is not supported during
    // iteration over it: https://lists.llvm.org/pipermail/llvm-dev/2011-May/039970.html
    std::vector<CallBase *> callsToWrap;

    for (auto &bblock : func) {
      for (auto &inst : bblock) {
        if (!isa<CallInst>(&inst) && !isa<InvokeInst>(&inst)) {
          continue;
        }

        auto origCall = dyn_cast<CallBase>(&inst);
        Function *callee = origCall->getCalledFunction();

        // TODO: Indirect calls
        if (callee) {
          StringRef name = callee->getName();
          char demangledName[DEMANGLED_LEN_MAX];

          // Prefixes for old and new (v0) Rust mangled names, Linux-only
          if (name.startswith("_ZN") || name.startswith("_R")) {
            char *mangledName = strdup(name.str().c_str());
            if (rustc_demangle(mangledName, demangledName, DEMANGLED_LEN_MAX) == 0) {
              free(mangledName);
              throw std::runtime_error("Demangling failed");
            }
            name = StringRef(demangledName);
            free(mangledName);
          }

          if (name.endswith(FUNC_SUFFIX)) {
            callsToWrap.push_back(origCall);
          }
        }
      }
    }

    LLVM_DEBUG(dbgs() << "Found " << callsToWrap.size() << " calls to wrap\n");

    for (auto *&origCall : callsToWrap) {
      Function *callee = origCall->getCalledFunction();

      // TODO: Indirect calls
      if (callee) {
        Type *typeParams[] = {callee->getFunctionType()->getPointerTo()};

        FunctionType *wrapperFuncType = FunctionType::get(
          Type::getVoidTy(func.getContext()),
          typeParams,
          false
        );
        FunctionCallee wrapperFunc = mod.getOrInsertFunction("wrapper", wrapperFuncType);

        Value *callParams[] = {callee};
        CallInst *wrapperCall = CallInst::Create(
          wrapperFunc,
          callParams,
          "",    // Not allowed to assign a name in this case
          static_cast<Instruction *>(nullptr)
        );
        ReplaceInstWithInst(origCall, wrapperCall);

        didWrap = true;
      }
    }
  }

  return didWrap;
}


PreservedAnalyses IndirectionPass::run(llvm::Module &m,
                                       llvm::ModuleAnalysisManager &) {
  bool Changed = runOnModule(m);

  return (Changed ? llvm::PreservedAnalyses::none()
                  : llvm::PreservedAnalyses::all());
}


extern "C" LLVM_ATTRIBUTE_WEAK ::llvm::PassPluginLibraryInfo
llvmGetPassPluginInfo() {
  return {
    LLVM_PLUGIN_API_VERSION,
    "indirection-pass",
    LLVM_VERSION_STRING,
    [](PassBuilder &pb) {
      pb.registerPipelineParsingCallback(
        [](StringRef name, ModulePassManager &mpm, ...) {
          if (name == "indirection-pass") {
            mpm.addPass(IndirectionPass());
            return true;
          }
          return false;
        }
      );
    }
  };
}
