#include "llvm/Passes/PassPlugin.h"
#include "llvm/Passes/PassBuilder.h"
#include "llvm/Transforms/Utils/BasicBlockUtils.h"

#include "IndirectionPass.h"


using namespace llvm;

#define DEBUG_TYPE "indirection-pass"


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
          if (callee->getName() == "foo") {
            callsToWrap.push_back(origCall);
          }
        }
      }
    }

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


PreservedAnalyses IndirectionPass::run(llvm::Module &M,
                                       llvm::ModuleAnalysisManager &) {
  bool Changed = runOnModule(M);

  return (Changed ? llvm::PreservedAnalyses::none()
                  : llvm::PreservedAnalyses::all());
}


extern "C" LLVM_ATTRIBUTE_WEAK ::llvm::PassPluginLibraryInfo
llvmGetPassPluginInfo() {
  return {
    LLVM_PLUGIN_API_VERSION,
    "indirection-pass",
    LLVM_VERSION_STRING,
    [](PassBuilder &PB) {
      PB.registerPipelineParsingCallback(
        [](StringRef Name, ModulePassManager &MPM, ...) {
          if (Name == "indirection-pass") {
            MPM.addPass(IndirectionPass());
            return true;
          }
          return false;
        }
      );
    }
  };
}
