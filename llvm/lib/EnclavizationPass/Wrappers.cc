#include "llvm/IR/DerivedTypes.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/InstrTypes.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/Value.h"

#include "EnclavizationPass/Export.h"
#include "EnclavizationPass/Names.h"
#include "EnclavizationPass/Wrappers.h"


using namespace llvm;


FunctionCallee getPregate(CallBase *callToWrap, EdlFile &edlFile) {
  Function *wrappedFunc = callToWrap->getCalledFunction();
  Module *mod = wrappedFunc->getParent();
  std::string pregateName = getPregateName(wrappedFunc);

  Function *pregateFunc = mod->getFunction(pregateName);
  if (pregateFunc) {
    // Gate has already been created (in previous calls)
    return pregateFunc;
  }

  FunctionType *wrappedType = wrappedFunc->getFunctionType();
  Type *gateReturnType = wrappedType->getReturnType();
  std::vector<Type *> gateArgTypes;
  for (auto argType = wrappedType->param_begin(); argType != wrappedType->param_end(); ++argType) {
    gateArgTypes.push_back(*argType);
  }
  FunctionType *pregateType = FunctionType::get(gateReturnType, gateArgTypes, false);

  {
    // This will always insert a new Function, because we already checked for existance above
    FunctionCallee callee = mod->getOrInsertFunction(pregateName, pregateType);
    pregateFunc = static_cast<Function *>(callee.getCallee());
  }

  // Adopt param and return attributes
  AttributeList wrappedAttrs = wrappedFunc->getAttributes();
  for (size_t i = 0; i < pregateFunc->arg_size(); ++i) {
    AttrBuilder builder = AttrBuilder(wrappedAttrs.getParamAttributes(i));
    pregateFunc->addParamAttrs(i, builder);
  }
  {
    AttrBuilder builder = AttrBuilder(wrappedAttrs.getRetAttributes());
    pregateFunc->addAttributes(AttributeList::ReturnIndex, builder);
  }

  FunctionCallee postgateFunc = getPostgate(callToWrap, edlFile);

  // New Function doesn't appear to have a real BasicBlock so far, getEntryBlock() only gives a sentinel
  BasicBlock *gateBlock = BasicBlock::Create(
    mod->getContext(),
    "call_postgate",
    pregateFunc,
    nullptr
  );

  std::vector<Value *> postgateArgs;
  for (auto arg = pregateFunc->arg_begin(); arg != pregateFunc->arg_end(); ++arg) {
    postgateArgs.push_back(arg);
  }
  CallInst *postgateCall = CallInst::Create(
    postgateFunc,
    postgateArgs,
    "",    // Not allowed to assign a name here
    gateBlock
  );

  // "[...] only one instance of a particular type is ever created. Thus seeing if two types are equal is a
  // matter of doing a trivial pointer comparison."
  if (wrappedFunc->getReturnType() == Type::getVoidTy(mod->getContext())) {
    ReturnInst::Create(
      mod->getContext(),
      gateBlock
    );
  } else {
    ReturnInst::Create(
      mod->getContext(),
      postgateCall,
      gateBlock
    );
  }

  return pregateFunc;
}


FunctionCallee getPostgate(CallBase *callToWrap, EdlFile &edlFile) {
  Function *wrappedFunc = callToWrap->getCalledFunction();
  Module *mod = wrappedFunc->getParent();
  std::string gateName = getPostgateName(wrappedFunc);

  Function *gateFunc = mod->getFunction(gateName);
  if (gateFunc) {
    // Gate has already been created (in previous calls)
    return gateFunc;
  }

  FunctionType *wrappedType = wrappedFunc->getFunctionType();
  Type *gateReturnType = wrappedType->getReturnType();
  std::vector<Type *> gateArgTypes;
  for (auto argType = wrappedType->param_begin(); argType != wrappedType->param_end(); ++argType) {
    gateArgTypes.push_back(*argType);
  }
  FunctionType *gateType = FunctionType::get(gateReturnType, gateArgTypes, false);

  {
    FunctionCallee callee = mod->getOrInsertFunction(gateName, gateType);
    gateFunc = static_cast<Function *>(callee.getCallee());
  }
  edlFile.addFunction(gateFunc);

  // Adopt param and return attributes
  AttributeList wrappedAttrs = wrappedFunc->getAttributes();
  for (size_t i = 0; i < gateFunc->arg_size(); ++i) {
    AttrBuilder builder = AttrBuilder(wrappedAttrs.getParamAttributes(i));
    gateFunc->addParamAttrs(i, builder);
  }
  {
    AttrBuilder builder = AttrBuilder(wrappedAttrs.getRetAttributes());
    gateFunc->addAttributes(AttributeList::ReturnIndex, builder);
  }

  BasicBlock *gateBlock = BasicBlock::Create(
    mod->getContext(),
    "call_enclaved",
    gateFunc,
    nullptr
  );

  std::vector<Value *> wrappedArgs;
  for (auto arg = gateFunc->arg_begin(); arg != gateFunc->arg_end(); ++arg) {
    wrappedArgs.push_back(arg);
  }
  CallInst *wrappedCall = CallInst::Create(
    wrappedFunc,
    wrappedArgs,
    "",    // Not allowed to assign a name here
    gateBlock
  );

  if (wrappedFunc->getReturnType() == Type::getVoidTy(mod->getContext())) {
    ReturnInst::Create(
      mod->getContext(),
      gateBlock
    );
  } else {
    ReturnInst::Create(
      mod->getContext(),
      wrappedCall,
      gateBlock
    );
  }

  return gateFunc;
}
