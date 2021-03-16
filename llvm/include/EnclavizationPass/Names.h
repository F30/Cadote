#ifndef ENCLAVIZATION_PASS_NAMES_H
#define ENCLAVIZATION_PASS_NAMES_H

#include <sstream>
#include <string>

#include "llvm/IR/Function.h"


#define TO_WRAP_SUFFIX "_enclaved_"
#define PRE_GATE_PREFIX "_enclave_pregate_"
#define POST_GATE_PREFIX "_enclave_postgate"


/*
 * Get name of a wrapper calling into the Enclave (but stil running outside of it).
 */
inline std::string getPregateName(llvm::Function *func) {
  std::stringstream gateName;
  gateName << PRE_GATE_PREFIX << func->getName().str();

  return gateName.str();
}

/*
 * Get name of a wrapper running inside the Enclave.
 */
inline std::string getPostgateName(llvm::Function *func) {
  std::stringstream entranceName;
  entranceName << POST_GATE_PREFIX << func->getName().str();

  return entranceName.str();
}

#endif
