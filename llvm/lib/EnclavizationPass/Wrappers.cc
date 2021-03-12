#include <sstream>
#include <string>

#include "llvm/IR/DerivedTypes.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/InstrTypes.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/Value.h"

#include "EnclavizationPass/Constants.h"
#include "EnclavizationPass/Wrappers.h"


using namespace llvm;


FunctionCallee getWrapper(CallBase *callToWrap) {
  Function *wrappedFunc = callToWrap->getCalledFunction();
  Module *mod = wrappedFunc->getParent();

  std::stringstream wrapperName;
  wrapperName << GATE_PREFIX << wrappedFunc->getName().str();

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
    "call_enclaved",
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
