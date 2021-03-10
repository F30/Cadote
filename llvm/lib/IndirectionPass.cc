#include <stdexcept>
#include <sstream>
#include <string>

#include "llvm/Passes/PassPlugin.h"
#include "llvm/Passes/PassBuilder.h"
#include "llvm/Transforms/Utils/BasicBlockUtils.h"

#include "rustc_demangle.h"
#include "IndirectionPass.h"


using namespace llvm;

#define DEBUG_TYPE "indirection-pass"

#define TO_WRAP_SUFFIX "_wrapped_"
#define WRAPPER_PREFIX "_indirection_wrapper_"
#define DEMANGLED_LEN_MAX 200


static FunctionCallee getWrapper(CallBase *callToWrap) {
  Function *wrappedFunc = callToWrap->getCalledFunction();
  Module *mod = wrappedFunc->getParent();

  std::stringstream wrapperName;
  wrapperName << WRAPPER_PREFIX << wrappedFunc->getName().str();

  Function *wrapperFunc = mod->getFunction(wrapperName.str());
  if (wrapperFunc) {
    // Wrapper has already been created (in previous calls)
    return wrapperFunc;
  }

  FunctionType *funcType = wrappedFunc->getFunctionType();
  // This will always insert a new Function, because we already checked for existance above
  FunctionCallee wrapperFuncCallee = mod->getOrInsertFunction(wrapperName.str(), funcType);
  wrapperFunc = static_cast<Function *>(wrapperFuncCallee.getCallee());

  // Adopt param and return attributes
  AttributeList wrappedAttrs = wrappedFunc->getAttributes();
  for (size_t i = 0; i < wrapperFunc->arg_size(); ++i) {
    AttrBuilder builder = AttrBuilder(wrappedAttrs.getParamAttributes(i));
    wrapperFunc->addParamAttrs(i, builder);
  } {
    AttrBuilder builder = AttrBuilder(wrappedAttrs.getRetAttributes());
    wrapperFunc->addAttributes(AttributeList::ReturnIndex, builder);
  }

  // New Function doesn't appear to have a real BasicBlock so far, getEntryBlock() only gives a sentinel
  BasicBlock *wrapperBlock = BasicBlock::Create(
    mod->getContext(),
    "call_wrapped",
    wrapperFunc,
    nullptr
  );

  std::vector<Value *> wrappedArgs;
  for (auto arg = wrapperFunc->arg_begin(); arg != wrapperFunc->arg_end(); ++arg) {
    wrappedArgs.push_back(arg);
  }
  CallInst *wrappedCall = CallInst::Create(
    wrappedFunc,
    wrappedArgs,
    "",    // Not allowed to assign a name here
    wrapperBlock
  );

  // "[...] only one instance of a particular type is ever created. Thus seeing if two types are equal is a
  // matter of doing a trivial pointer comparison."
  if (wrappedFunc->getReturnType() == Type::getVoidTy(mod->getContext())) {
    ReturnInst::Create(
      mod->getContext(),
      wrapperBlock
    );
  } else {
    ReturnInst::Create(
      mod->getContext(),
      wrappedCall,
      wrapperBlock
    );
  }

  return wrapperFunc;
}


bool IndirectionPass::runOnModule(Module &mod) {
  bool didWrap = false;

  // First generate list of calls to be wrapped in order to not change a Module, Function or BasicBlock
  // while iterating over it
  std::vector<CallBase *> callsToWrap;

  for (auto &func : mod) {
    if (func.isDeclaration()) {
      continue;
    }

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

          if (name.endswith(TO_WRAP_SUFFIX)) {
            callsToWrap.push_back(origCall);
          }
        }
      }
    }
  }

  LLVM_DEBUG(dbgs() << "Found " << callsToWrap.size() << " calls to wrap\n");

  for (auto *&origCall : callsToWrap) {
    // TODO: Indirect calls
    if (origCall->getCalledFunction()) {
      std::vector<Value *> callArgs;
      for (auto arg = origCall->arg_begin(); arg != origCall->arg_end(); ++arg) {
        callArgs.push_back(arg->get());
      }

      FunctionCallee wrapperFunc = getWrapper(origCall);
      CallInst *wrapperCall = CallInst::Create(
        wrapperFunc,
        callArgs,
        "",    // Not allowed to assign a name here
        static_cast<Instruction *>(nullptr)
      );
      ReplaceInstWithInst(origCall, wrapperCall);

      didWrap = true;
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
