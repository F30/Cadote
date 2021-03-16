#ifndef ENCLAVIZATION_PASS_WRAPPERS_H
#define ENCLAVIZATION_PASS_WRAPPERS_H

#include "llvm/IR/DerivedTypes.h"
#include "llvm/IR/InstrTypes.h"


llvm::FunctionCallee getPregate(llvm::CallBase *, EdlFile &);

llvm::FunctionCallee getPostgate(llvm::CallBase *, EdlFile &);


#endif
