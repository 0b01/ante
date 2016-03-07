#ifndef COMPILER_H
#define COMPILER_H

#include <climits> //required by llvm is using clang
#include <llvm/IR/LegacyPassManager.h>
#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/Module.h>
#include <memory>
#include <stack>
#include <map>

using namespace llvm;
using namespace std;

/* Forward-declarations of Nodes defined in parser.h */
struct Node;
struct VarNode;
struct TypeNode;
struct BinOpNode;
struct StrLitNode;
struct IntLitNode;
struct FuncDeclNode;
struct FuncCallNode;
struct DataDeclNode;


/*
 *  Used for storage of additional information, such as signedness,
 *  not represented by llvm::Type
 */
struct TypedValue {
    Value *val;
    int type;

    TypedValue(Value *v, int ty) : val(v), type(ty){}
};


struct Variable {
    string name;
    TypedValue *tval;
    unsigned int scope;


    Value* getVal() const{
        return tval->val;
    }
   
    int getType() const{
        return tval->type;
    }

    bool isPtr() const{
        return tval->type == '*';
    }

    Variable(string n, TypedValue *tv, unsigned int s) : name(n), tval(tv), scope(s){}
};


namespace ante{
    struct Compiler{
        unique_ptr<legacy::FunctionPassManager> passManager;
        unique_ptr<Module> module;
        unique_ptr<Node> ast;
        IRBuilder<> builder;

        //Stack of maps of variables mapped to their identifier.
        //Maps are seperated according to their scope.
        stack<std::map<string, Variable*>> varTable;

        //Map of declared, but non-defined functions
        map<string, FuncDeclNode*> fnDecls;

        //Map of declared usertypes
        map<string, DataDeclNode*> userTypes;

        bool errFlag, compiled;
        string fileName;
        unsigned int scope;
        
        Compiler(char *fileName);
        ~Compiler();

        void compile();
        void compileNative();
        void compilePrelude();
        void emitIR();
        void enterNewScope();
        void exitScope();
        
        TypedValue* compAdd(TypedValue *l, TypedValue *r, BinOpNode *op);
        TypedValue* compSub(TypedValue *l, TypedValue *r, BinOpNode *op);
        TypedValue* compMul(TypedValue *l, TypedValue *r, BinOpNode *op);
        TypedValue* compDiv(TypedValue *l, TypedValue *r, BinOpNode *op);
        TypedValue* compRem(TypedValue *l, TypedValue *r, BinOpNode *op);
        TypedValue* compGEP(TypedValue *l, TypedValue *r, BinOpNode *op);
        
        TypedValue* compErr(string msg, unsigned int row, unsigned int col);

        Function* getFunction(string& name);
        Function* compFn(FuncDeclNode *fn);
        void registerFunction(FuncDeclNode *func);

        unsigned int getScope() const;
        Variable* lookup(string var) const;
        void stoVar(string var, Variable *val);
        DataDeclNode* lookupType(string tyname) const;
        void stoType(DataDeclNode *ty);

        static bool isSigned(Node *n);
        void checkIntSize(TypedValue **lhs, TypedValue **rhs);
        
        static Type* typeNodeToLlvmType(TypeNode *tyNode);
        static Type* tokTypeToLlvmType(int tokTy, string typeName);
        static int llvmTypeToTokType(Type *t);
        static bool llvmTypeEq(Type *l, Type *r);

        static size_t getTupleSize(Node *tup);
        static char getBitWidthOfTokTy(int tokTy);
        static bool isUnsignedTokTy(int tokTy);
        
        static int compileIRtoObj(Module *m, string inFile, string outFile);
        static int linkObj(string inFiles, string outFile);
    };
}


#endif
