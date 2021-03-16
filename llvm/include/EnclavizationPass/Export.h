#ifndef ENCLAVIZATION_PASS_EXPORT_H
#define ENCLAVIZATION_PASS_EXPORT_H

#include <fstream>
#include <string>

#include "llvm/IR/Argument.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/Type.h"

#include "EnclavizationPass/Export.h"


class EdlFile {
  public:
    EdlFile(std::string path);
    ~EdlFile();
    void addFunction(llvm::Function *);

   private:
     std::string getEdlType(llvm::Type *, llvm::LLVMContext &);
     std::vector<std::string> getEdlAttributes(llvm::Argument *);

     std::ofstream fstream;
};


#endif
