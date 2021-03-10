#ifndef CODE_COMP_INDIRECTION_PASS_H
#define CODE_COMP_INDIRECTION_PASS_H

#include "llvm/IR/PassManager.h"
#include "llvm/Pass.h"

struct IndirectionPass : public llvm::PassInfoMixin<IndirectionPass> {	
  public:
    llvm::PreservedAnalyses run(llvm::Module &M, llvm::ModuleAnalysisManager &);
    bool runOnModule(llvm::Module &M);
};

#endif
