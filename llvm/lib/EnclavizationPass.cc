#include <sstream>
#include <string>

#include "llvm/ADT/StringRef.h"
#include "llvm/IR/DerivedTypes.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/InstrTypes.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/PassManager.h"
#include "llvm/IR/Value.h"
#include "llvm/Passes/PassPlugin.h"
#include "llvm/Passes/PassBuilder.h"
#include "llvm/Support/Debug.h"
#include "llvm/Transforms/Utils/BasicBlockUtils.h"

#include "rustc_demangle.h"
#include "EnclavizationPass.h"
#include "EnclavizationPass/Constants.h"
#include "EnclavizationPass/Wrappers.h"


using namespace llvm;

#define DEBUG_TYPE "enclavization-pass"

#define DEMANGLED_LEN_MAX 200


bool EnclavizationPass::runOnModule(Module &mod) {
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


PreservedAnalyses EnclavizationPass::run(llvm::Module &m, llvm::ModuleAnalysisManager &) {
  bool changed = runOnModule(m);
  return (changed ? llvm::PreservedAnalyses::none()
                  : llvm::PreservedAnalyses::all());
}


extern "C" LLVM_ATTRIBUTE_WEAK ::llvm::PassPluginLibraryInfo llvmGetPassPluginInfo() {
  return {
    LLVM_PLUGIN_API_VERSION,
    "enclavization-pass",
    LLVM_VERSION_STRING,
    [](PassBuilder &pb) {
      pb.registerPipelineParsingCallback(
        [](StringRef name, ModulePassManager &mpm, ...) {
          if (name == "enclavization-pass") {
            mpm.addPass(EnclavizationPass());
            return true;
          }
          return false;
        }
      );
    }
  };
}
