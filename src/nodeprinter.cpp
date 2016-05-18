#include "parser.h"
using namespace ante::parser;


inline void maybePrintArr(Node *n){
    if(n){
        cout << ", ";
        n->print();
    }
}

/*
 *  Prints a list of nodes, can print
 *  entire parse tree if passed the root.
 */
void parser::printBlock(Node *block){
    while(block){
        block->print();
        block = block->next.get();
        cout << endl;
    }
}

void IntLitNode::print(){
    cout << val;
    maybePrintArr(next.get());
}

void FltLitNode::print(){
    cout << val;
    maybePrintArr(next.get());
}

void BoolLitNode::print(){
    if(val)
        cout << "true";
    else
        cout << "false";
    maybePrintArr(next.get());
}

void StrLitNode::print(){
    cout << '"' << val << '"';
    maybePrintArr(next.get());
}

void ArrayNode::print(){
    putchar('[');
    /*for(size_t i = 0; i < exprs.size(); i++){
        exprs[i]->print();
        if(i != exprs.size() - 1){
            cout << ", ";
        }
    }*/
    exprs[0]->print();
    putchar(']');
}

void TupleNode::print(){
    putchar('(');
    if(exprs.size() > 0)
        exprs[0]->print();
    putchar(')');
}

void ModNode::print(){
    Lexer::printTok(mod);
}

void TypeNode::print(){
    cout << typeTagToStr(type);
}

void TypeCastNode::print(){
    putchar('(');
    typeExpr->print();
    putchar(':');
    putchar(' ');
    rval->print();
    putchar(')');
    maybePrintArr(next.get());
}

void UnOpNode::print(){
    putchar('(');
    Lexer::printTok(op);
    putchar(' ');
    rval->print();
    putchar(')');
    maybePrintArr(next.get());
}

void BinOpNode::print(){
    putchar('(');
    if(lval) lval->print();
    putchar(' ');
    Lexer::printTok(op);
    putchar(' ');
    if(rval) rval->print();
    putchar(')');

    maybePrintArr(next.get());
}

void RetNode::print(){
    cout << "return ";
    if(expr) expr->print();
}

void ImportNode::print(){
    cout << "import ";
    expr->print();
}

void IfNode::print(){
    if(condition.get()){
        cout << "if ";
        condition->print();
        puts(" then");
        printBlock(child.get());
   
        //If this if/elif has an else/elif, print it.
        if(elseN.get()){
            cout << "el";
            elseN->print();
        }else{
            cout << "endif\n";
        }
    }else{
        cout << "se\n"; //This ifnode is an elsenode
        printBlock(child.get());
        cout << "endif\n";
        //else nodes should never have additional
        //ifnodes in elseN, so dont bother checking.
    }
}

//unlike IfNodes, an ExprIfNode's
//condition, thenN, and elseN are all
//guarenteed to be initialized
void ExprIfNode::print(){
    cout << "if ";
    condition->print();
    puts(" then");
    thenN->print();
    puts("else");
    elseN->print();
}

void NamedValNode::print(){
    typeExpr->print();
    putchar(' ');
    cout << name;

    maybePrintArr(next.get());
}

void VarNode::print(){
    cout << name;
    maybePrintArr(next.get());
}

void RefVarNode::print(){
    cout << "(ref " << name << ")";
    maybePrintArr(next.get());
}

void FuncCallNode::print(){
    cout << name;
    params->print();
}

void LetBindingNode::print(){
    cout << "let ";
    if(typeExpr.get()){
        typeExpr->print();
    }
    cout << ' ' << name << " = ";
    expr->print(); //expr is not null-checked since it is required to be non-null
    putchar('\n');
}

void VarDeclNode::print(){
    cout << "varDecl ";
    if(typeExpr) typeExpr->print();
    cout << ' ' << name << " = ";
    if(expr) expr->print();
    else cout << "(undef)";
}

void VarAssignNode::print(){
    cout << "varAssign ";
    if(ref_expr) ref_expr->print();
    cout << " = ";
    if(expr) expr->print();
    else cout << "(undef)";
}

void ExtNode::print(){
    cout << "ext ";
    typeExpr->print();
    cout << "\n";
    printBlock(methods.get());
    puts("end ext");
}

void WhileNode::print(){
    cout << "while ";
    condition->print();
    puts(" do");
    printBlock(child.get());
    puts("end while");
}

void FuncDeclNode::print(){
    cout << "fnDecl ";
    type->print();
    cout << ' ' << name << ": ";
    if(params) params->print();
    puts("\nfnbody:");
    printBlock(child.get());
    puts("endfn");
}

void DataDeclNode::print(){
    cout << "data " << name << endl;
    printBlock(child.get());
}
