#ifndef ENCLAVIZATION_PASS_H
#define ENCLAVIZATION_PASS_H

#include "llvm/IR/PassManager.h"
#include "llvm/Pass.h"


struct EnclavizationPass : public llvm::PassInfoMixin<EnclavizationPass> {
  public:
    llvm::PreservedAnalyses run(llvm::Module &M, llvm::ModuleAnalysisManager &);
    bool runOnModule(llvm::Module &M);
};


#endif
