// This file must be explicitly converted to a `parser_expanded.rs` by `rustylr`

use lua_tokenizer::Token;
use lua_tokenizer::TokenType;

use crate::expression;
use crate::Expression;
use crate::statement;
use crate::Statement;
use crate::Span;
use crate::SpannedString;
use crate::ParseError;
use crate::ChunkOrExpressions;

macro_rules! new_binary_node {
    ($variant:ident, $op:ident, $lhs:ident, $rhs:ident) => {{
        let span = $lhs.span().merge_ordered(&$rhs.span());
        let span_op = $op.span();
        let binary_data = expression::ExprBinaryData::new($lhs, $rhs, span, span_op);
        Expression::Binary(
            expression::ExprBinary::$variant(
                binary_data
            )
        )
    }};
}
macro_rules! new_unary_node {
    ($variant:ident, $op:ident, $lhs:ident) => {{
        let span = $op.span().merge_ordered(&$lhs.span());
        let span_op = $op.span();
        let unary_data = expression::ExprUnaryData::new($lhs, span, span_op);
        Expression::Unary(
            expression::ExprUnary::$variant(
                unary_data
            )
        )
    }};
}

fn filter( token: &Token ) -> &TokenType {
    &token.token_type
}

// @TODO Block span

%%

%glr;
%lalr;
%tokentype Token;
%err ParseError;
%filter filter;


%token ident TokenType::Ident(_);

%token string_literal TokenType::String(_);
%token numeric_literal TokenType::Numeric(_);
%token nil TokenType::Nil;
%token bool_ TokenType::Bool(_);

%token plus TokenType::Plus;
%token minus TokenType::Minus;
%token asterisk TokenType::Asterisk;
%token slash TokenType::Slash;
%token percent TokenType::Percent;
%token caret TokenType::Caret;
%token hash TokenType::Hash;
%token ampersand TokenType::Ampersand;
%token tilde TokenType::Tilde;
%token pipe TokenType::Pipe;
%token lessless TokenType::LessLess;
%token greatergreater TokenType::GreaterGreater;
%token slashslash TokenType::SlashSlash;
%token equalequal TokenType::EqualEqual;
%token tildeequal TokenType::TildeEqual;
%token lessequal TokenType::LessEqual;
%token greaterequal TokenType::GreaterEqual;
%token less TokenType::Less;
%token greater TokenType::Greater;
%token equal TokenType::Equal;
%token lparen TokenType::LParen;
%token rparen TokenType::RParen;
%token lbrace TokenType::LBrace;
%token rbrace TokenType::RBrace;
%token lbracket TokenType::LBracket;
%token rbracket TokenType::RBracket;
%token coloncolon TokenType::ColonColon;
%token semicolon TokenType::Semicolon;
%token colon TokenType::Colon;
%token comma TokenType::Comma;
%token dot TokenType::Dot;
%token dotdot TokenType::DotDot;
%token dotdotdot TokenType::DotDotDot;

%token and_ TokenType::And;
%token break_ TokenType::Break;
%token do_ TokenType::Do;
%token else_ TokenType::Else;
%token elseif_ TokenType::Elseif;
%token end_ TokenType::End;
%token for_ TokenType::For;
%token function_ TokenType::Function;
%token goto_ TokenType::Goto;
%token if_ TokenType::If;
%token in_ TokenType::In;
%token local_ TokenType::Local;
%token not_ TokenType::Not;
%token or_ TokenType::Or;
%token repeat_ TokenType::Repeat;
%token return_ TokenType::Return;
%token then_ TokenType::Then;
%token until_ TokenType::Until;
%token while_ TokenType::While;

%start ChunkOrExpressions;

ChunkOrExpressions(ChunkOrExpressions)
    : Chunk { ChunkOrExpressions::Chunk(Chunk) }
    | ExpList1 { ChunkOrExpressions::Expressions(ExpList1) }
    ;

Chunk(statement::Block)
    : Block
    ;

Block(statement::Block)
    : Statement* ReturnStatement? {
        let span0 = if let Some(first) = Statement.first() {
            first.span()
        } else {
            Span::new_none()
        };
        if let Some(ret) = ReturnStatement {
            let span1 = ret.span();
            let span = span0.merge_ordered(&span1);
            statement::Block::new( Statement, Some(ret), span )
        } else {
            let span1 = if let Some(last) = Statement.last() {
                last.span()
            } else {
                Span::new_none()
            };
            let span = span0.merge_ordered(&span1);
            statement::Block::new( Statement, None, span )
        }
    }
    ;

Statement(Statement)
    : semicolon { Statement::None( statement::StmtNone::new(semicolon.span()) ) }
    | VarList equal ExpList1 {
        let span = VarList.first().unwrap().span().merge_ordered(&ExpList1.last().unwrap().span());
        let span_eq = equal.span();
        Statement::Assignment( statement::StmtAssignment::new(VarList, ExpList1, span, span_eq) )
    }
    | FunctionCall {
        Statement::FunctionCall(
            FunctionCall
        )
    }
    | c1=coloncolon ident c2=coloncolon {
        let span = c1.span().merge_ordered(&c2.span());
        Statement::Label( statement::StmtLabel::new(
            ident.into(),
            span
        ))
    }
    | break_ {
        Statement::Break( statement::StmtBreak::new( break_.span() ) )
    }
    | goto_ ident {
        let span = goto_.span().merge_ordered( &ident.span() );
        Statement::Goto( statement::StmtGoto::new(ident.into(), span) )
    }
    | do_ Block end_ {
        let span = do_.span().merge_ordered( &end_.span() );
        Statement::Do( statement::StmtDo::new(Block, span) )
    }
    | while_ Exp do_! Block end_ {
        let span = while_.span().merge_ordered( &end_.span() );
        Statement::While( statement::StmtWhile::new(Exp, Block, span) )
    }
    | repeat_ Block until_! Exp {
        let span = repeat_.span().merge_ordered( &Exp.span() );
        Statement::Repeat( statement::StmtRepeat::new(Block, Exp, span) )
    }
    | if_ Exp then_! Block elseifs=ElseIf* else_=(else_! Block)? end_ {
        let span = if_.span().merge_ordered( &end_.span() );
        Statement::If(
            statement::StmtIf::new(
                Exp,
                Block,
                elseifs,
                else_,
                span
            )
        )
    }
    | for_ ident equal! start=Exp comma! end=Exp step=(comma! Exp)? do_! Block end_ {
        let span = for_.span().merge_ordered( &end_.span() );
        Statement::For(
            statement::StmtFor::new(
                ident.token_type.into_ident().unwrap(),
                start,
                end,
                step.unwrap_or_else(|| {
                    // @TODO no none span
                    Expression::Numeric(expression::ExprNumeric::new(1.into(), Span::new_none()))
                }),
                Block,
                span,
            )
        )
    }
    | for_ NameList in_! ExpList1 do_! Block end_ {
        let span = for_.span().merge_ordered( &end_.span() );
        Statement::ForGeneric( statement::StmtForGeneric::new(NameList, ExpList1, Block, span) )
    }
    | function_ FuncName FuncBody {
        let span = function_.span().merge_ordered( &FuncBody.span() );
        Statement::FunctionDefinition(
            statement::StmtFunctionDefinition::new(FuncName, FuncBody, span)
        )
    }
    | local_ function_! ident FuncBody {
        let span = local_.span().merge_ordered( &FuncBody.span() );
        Statement::FunctionDefinitionLocal(
            statement::StmtFunctionDefinitionLocal::new(ident.into(), FuncBody, span)
        )
    }
    | local_ AttNameList rhs_list=(equal! ExpList1)? {
        let span0 = local_.span();
        if let Some(rhs) = rhs_list {
            let span = span0.merge_ordered( &rhs.last().unwrap().span() );
            Statement::LocalDeclaration( statement::StmtLocalDeclaration::new(AttNameList, Some(rhs), span) )
        } else {
            let span = AttNameList.last().unwrap().span();
            Statement::LocalDeclaration( statement::StmtLocalDeclaration::new(AttNameList, None, span) )
        }
    }
    ;

ElseIf(statement::StmtElseIf)
    : elseif_ Exp then_ Block {
        let span = if Block.span().is_none() {
            elseif_.span().merge_ordered( &then_.span() )
        } else {
            elseif_.span().merge_ordered( &Block.span() )
        };
        statement::StmtElseIf::new(Exp, Block, span)
    }
    ;

ReturnStatement(statement::ReturnStatement)
    : return_ ExpList0 semicolon? {
        let span0 = return_.span();
        let span = if let Some(last) = semicolon {
            span0.merge_ordered(&last.span())
        } else {
            if let Some(last) = ExpList0.last() {
                span0.merge_ordered(&last.span())
            } else {
                span0
            }
        };
        statement::ReturnStatement::new(ExpList0, span)
    }
    ;

Var(Expression)
    : ident {
        Expression::Ident( ident.into() )
    }
    | PrefixExp lbracket! Exp rbracket {
        let span = PrefixExp.span().merge_ordered(&rbracket.span());
        Expression::TableIndex( expression::ExprTableIndex::new(PrefixExp, Exp, span) )
    }
    | PrefixExp dot! ident {
        // a.b => a["b"]

        let span = PrefixExp.span().merge_ordered(&ident.span());
        let member = expression::ExprString::from(ident);

        Expression::TableIndex(
            expression::ExprTableIndex::new(
                PrefixExp,
                Expression::String(member),
                span
            )
        )
    }
    ;

PrefixExp(Expression)
    : Var
    | FunctionCall {
        Expression::FunctionCall(
            FunctionCall
        )
    }
    | lparen! Exp rparen!
    ;


FunctionCall(expression::ExprFunctionCall)
    : PrefixExp Args {
        let span = PrefixExp.span().merge_ordered(&Args.span());
        expression::ExprFunctionCall::new( PrefixExp, None, Args, span )
    }
    | PrefixExp colon! ident Args {
        let span = PrefixExp.span().merge_ordered(&Args.span());
        expression::ExprFunctionCall::new( PrefixExp, Some(ident.into()), Args, span )
    }
    ;

Args(expression::FunctionCallArguments)
    : lparen ExpList0 rparen {
        let span = lparen.span().merge_ordered(&rparen.span());
        expression::FunctionCallArguments::new( ExpList0, span )
    }
    | TableConstructor {
        let span = TableConstructor.span();
        let table_expr = Expression::Table( TableConstructor );
        let exprs = vec![table_expr];
        expression::FunctionCallArguments::new( exprs, span )
    }
    | string_literal {
        let span = string_literal.span();
        let exprs = vec![Expression::String(
            string_literal.into()
        )];
        expression::FunctionCallArguments::new( exprs, span )
    }
    ;

// one or more comma-separated Vars
VarList(Vec<Expression>)
    : VarList comma Var {
        VarList.push(Var);
        VarList
    }
    | Var { vec![Var] }
    ;

// one or more comma-separated expressions
ExpList1(Vec<Expression>)
    : ExpList1 comma Exp {
        ExpList1.push(Exp);
        ExpList1
    }
    | Exp {
        vec![Exp]
    }
    ;
// zero or more comma-separated expressions
ExpList0(Vec<Expression>)
    : ExpList1 {
        ExpList1
    }
    | {
        vec![]
    }
    ;

// one or more comma-separated names
NameList(Vec<SpannedString>)
    : NameList comma! ident {
        NameList.push(ident.into());
        NameList
    }
    | ident {
        vec![ident.into()]
    }
    ;

AttName(statement::AttName)
    : ident Attrib {
        let span = ident.span();
        statement::AttName::new( ident.into(), Attrib, span )
    }
    ;
AttNameList(Vec<statement::AttName>)
    : AttNameList comma! AttName {
        AttNameList.push( AttName );
        AttNameList
    }
    | AttName {
        vec![ AttName ]
    }
    ;

Attrib(Option<statement::Attrib>)
    : less! ident greater! {
        let s:SpannedString = ident.into();
        match s.as_str() {
            "const" => Some(statement::Attrib::Const),
            "close" => Some(statement::Attrib::Close),
            _ => {
                return Err( ParseError::UnknownAttribute(s) );
            }
        }
    }
    | { None }
    ;

%left or_;
%left and_;
%left less lessequal greater greaterequal tildeequal equalequal;
%left pipe;
%left tilde;
%left ampersand;
%left lessless greatergreater;
%right dotdot;
%left plus minus;
%left asterisk slash slashslash percent;
%right caret;
%precedence UNOT UHASH UMINUS UPLUS UTILDE;

%precedence PREFIX;
// %precedence lparen;

Exp(Expression)
    : numeric_literal {
        Expression::Numeric(
            numeric_literal.into()
        )
    }
    | nil {
        Expression::Nil( nil.into() )
    }
    | string_literal {
        Expression::String(
            string_literal.into()
        )
    }
    | bool_ {
        Expression::Bool(
            bool_.into()
        )
    }
    | dotdotdot {
        Expression::Variadic( dotdotdot.into() )
    }
    | FunctionDef {
        Expression::Function( FunctionDef )
    }
    | PrefixExp %prec PREFIX
    | TableConstructor {
        Expression::Table( TableConstructor )
    }
    | not_ Exp %prec UNOT {
        new_unary_node!(LogicalNot, not_, Exp)
    }
    | hash Exp %prec UHASH {
        new_unary_node!(Length, hash, Exp)
    }
    | minus Exp %prec UMINUS {
        new_unary_node!(Minus, minus, Exp)
    }
    | plus Exp %prec UPLUS {
        new_unary_node!(Plus, plus, Exp)
    }
    | tilde Exp %prec UTILDE {
        new_unary_node!(BitwiseNot, tilde, Exp)
    }
    | lhs=Exp asterisk rhs=Exp {
        new_binary_node!(Mul, asterisk, lhs, rhs)
    }
    | lhs=Exp slash rhs=Exp {
        new_binary_node!(Div, slash, lhs, rhs)
    }
    | lhs=Exp slashslash rhs=Exp {
        new_binary_node!(FloorDiv, slashslash, lhs, rhs)
    }
    | lhs=Exp percent rhs=Exp {
        new_binary_node!(Mod, percent, lhs, rhs)
    }
    | lhs=Exp plus rhs=Exp {
        new_binary_node!(Add, plus, lhs, rhs)
    }
    | lhs=Exp minus rhs=Exp {
        new_binary_node!(Sub, minus, lhs, rhs)
    }
    // right associative for concat '..'
    | lhs=Exp dotdot rhs=Exp {
        new_binary_node!(Concat, dotdot, lhs, rhs)
    }
    | lhs=Exp lessless rhs=Exp {
        new_binary_node!(ShiftLeft, lessless, lhs, rhs)
    }
    | lhs=Exp greatergreater rhs=Exp {
        new_binary_node!(ShiftRight, greatergreater, lhs, rhs)
    }
    | lhs=Exp ampersand rhs=Exp {
        new_binary_node!(BitwiseAnd, ampersand, lhs, rhs)
    }
    | lhs=Exp tilde rhs=Exp {
        new_binary_node!(BitwiseXor, tilde, lhs, rhs)
    }
    | lhs=Exp pipe rhs=Exp {
        new_binary_node!(BitwiseOr, pipe, lhs, rhs)
    }
    | lhs=Exp less rhs=Exp {
        new_binary_node!(LessThan, less, lhs, rhs)
    }
    | lhs=Exp lessequal rhs=Exp {
        new_binary_node!(LessEqual, lessequal, lhs, rhs)
    }
    | lhs=Exp greater rhs=Exp {
        new_binary_node!(GreaterThan, greater, lhs, rhs)
    }
    | lhs=Exp greaterequal rhs=Exp {
        new_binary_node!(GreaterEqual, greaterequal, lhs, rhs)
    }
    | lhs=Exp tildeequal rhs=Exp {
        new_binary_node!(NotEqual, tildeequal, lhs, rhs)
    }
    | lhs=Exp equalequal rhs=Exp {
        new_binary_node!(Equal, equalequal, lhs, rhs)
    }
    | lhs=Exp and_ rhs=Exp {
        new_binary_node!(LogicalAnd, and_, lhs, rhs)
    }
    | lhs=Exp or_ rhs=Exp {
        new_binary_node!(LogicalOr, or_, lhs, rhs)
    }
    | lhs=Exp caret rhs=Exp {
        new_binary_node!(Pow, caret, lhs, rhs)
    }
    ;


TableConstructor(expression::ExprTable)
    : lbrace FieldList rbrace {
        let span = lbrace.span().merge_ordered(&rbrace.span());
        expression::ExprTable::new( FieldList, span )
    }
    ;


// zero or more separated Fields, with optional trailing separator
FieldList(Vec<expression::TableField>)
    : $sep( Field, FieldSep, * );

Field(expression::TableField)
    : lbracket k=Exp rbracket! equal! v=Exp {
        let span = lbracket.span().merge_ordered(&v.span());
        expression::TableField::KeyValue(
            expression::TableFieldKeyValue::new(k, v, span)
        )
    }
    | ident equal! Exp {
        let span = ident.span().merge_ordered(&Exp.span());
        expression::TableField::NameValue(
            expression::TableFieldNameValue::new(ident.into(), Exp, span)
        )
    }
    | Exp {
        expression::TableField::Value(
            expression::TableFieldValue::new(Exp)
        )
    }
    ;

FieldSep: comma | semicolon ;


FunctionDef(expression::ExprFunction)
    : function_ FuncBody {
        let span = function_.span().merge_ordered(&FuncBody.span());
        FuncBody.span = span;
        FuncBody
    }
    ;

FuncBody(expression::ExprFunction)
    : lparen ParList? rparen! Block end_ {
        let span = lparen.span().merge_ordered(&end_.span());
        expression::ExprFunction::new(ParList, Block, span)
    }
    ;

// dot chained ident
FuncName1(Vec<SpannedString>)
    : FuncName1 dot ident {
        FuncName1.push( ident.into() );
        FuncName1
    }
    | ident {
        vec![ident.into()]
    }
    ;

FuncName(statement::FunctionName)
    : FuncName1 colon! ident {
        let span = FuncName1.first().unwrap().span().merge_ordered(&ident.span());
        statement::FunctionName::new( FuncName1, Some(ident.into()), span )
    }
    | FuncName1 {
        let span = FuncName1.first().unwrap().span().merge_ordered(
            &FuncName1.last().unwrap().span()
        );
        statement::FunctionName::new( FuncName1, None, span )
    }
    ;

ParList(expression::ParameterList)
    : NameList var=(comma! dotdotdot)? {
        if let Some(var) = var {
            let span = NameList.first().unwrap().span().merge_ordered(&var.span());
            expression::ParameterList::new( NameList, true, span )
        } else {
            let span = NameList.first().unwrap().span();
            expression::ParameterList::new( NameList, false, span )
        }
    }
    | dotdotdot {
        expression::ParameterList::new( Vec::new(), true, dotdotdot.span() )
    }
    ;