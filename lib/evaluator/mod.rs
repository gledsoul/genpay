pub mod object;
pub mod environment;

use std::collections::HashMap;
use parser::ast::*;
use evaluator::object::*;
use evaluator::environment::*;

pub struct Evaluator {
    env: Environment
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            env: Environment::new()
        }
    }

    pub fn eval_program(&mut self, prog: Program) -> Object {
        let return_data = self.eval_blockstmt(prog);
        match return_data {
            Object::ReturnValue(box v) => v,
            o => o,
        }
    }

    pub fn eval_blockstmt(&mut self, prog: BlockStmt) -> Object {
        match prog[..] {
            [] => Object::Null,
            [ref s] => self.eval_statement(s.clone()),
            [ref s, ref ss..] => {
                let object = self.eval_statement(s.clone());
                if object.is_returned() {
                    object
                } else {
                    self.eval_blockstmt(ss.to_vec())
                }
            },
        }
    }

    pub fn eval_statement(&mut self, stmt: Stmt) -> Object {
        match stmt {
            Stmt::ExprStmt(expr) => self.eval_expr(expr),
            Stmt::ReturnStmt(expr) => Object::ReturnValue(box self.eval_expr(expr)),
            Stmt::LetStmt(ident, expr) => {
                let object = self.eval_expr(expr);
                self.register_ident(ident, object)
            },
        }
    }

    pub fn register_ident(&mut self, ident: Ident, object: Object) -> Object {
        let Ident(name) = ident;
        self.env.set(&name, &object);
        object
    }

    pub fn eval_expr(&mut self, expr: Expr) -> Object {
        match expr {
            Expr::IdentExpr(i) => self.eval_ident(i),
            Expr::LitExpr(l) => self.eval_literal(l),
            Expr::PrefixExpr(prefix, box expr) => self.eval_prefix(prefix, expr),
            Expr::InfixExpr(infix, box expr1, box expr2) => self.eval_infix(infix, expr1, expr2),
            Expr::IfExpr { cond: box cond, consequence: conse, alternative: maybe_alter } =>
                self.eval_if(cond, conse, maybe_alter),
            Expr::FnExpr { params, body } => self.eval_fn(params, body),
            Expr::CallExpr { function: box fn_exp, arguments } =>
                self.eval_call(fn_exp, arguments),
            Expr::ArrayExpr(exprs) => self.eval_array(exprs),
            Expr::HashExpr(hash_exprs) => self.eval_hash(hash_exprs),
            Expr::IndexExpr { box array, box index } => self.eval_index(array, index),
        }
    }

    pub fn eval_ident(&mut self, ident: Ident) -> Object {
        let Ident(name) = ident;
        let var = self.env.get(&name);
        match var {
            Some(o) => (*o).clone(),
            None => Object::Error(format!("identifier not found: {}", name))
        }
    }

    pub fn eval_literal(&mut self, literal: Literal) -> Object {
        match literal {
            Literal::IntLiteral(i) => Object::Integer(i),
            Literal::BoolLiteral(b) => Object::Boolean(b),
            Literal::StringLiteral(s) => Object::String(s),
        }
    }

    pub fn eval_prefix(&mut self, prefix: Prefix, expr: Expr) -> Object {
        let object = self.eval_expr(expr);
        match prefix {
            Prefix::Not => {
                match self.otb(object) {
                    Ok(b) => Object::Boolean(!b),
                    Err(err) => err,
                }
            },
            Prefix::PrefixPlus => {
                match self.oti(object) {
                    Ok(i) => Object::Integer(i),
                    Err(err) => err,
                }
            },
            Prefix::PrefixMinus => {
                match self.oti(object) {
                    Ok(i) => Object::Integer(-i),
                    Err(err) => err,
                }
            },
        }
    }

    pub fn eval_infix(&mut self, infix: Infix, expr1: Expr, expr2: Expr) -> Object {
        let object1 = self.eval_expr(expr1);
        let object2 = self.eval_expr(expr2);
        match infix {
            Infix::Plus => self.object_add(object1, object2),
            Infix::Minus => {
                let i1 = self.oti(object1);
                let i2 = self.oti(object2);
                match (i1, i2) {
                    (Ok(i1), Ok(i2)) => Object::Integer(i1 - i2),
                    (Err(err), _) => err,
                    (_, Err(err)) => err,
                }
            },
            Infix::Divide => {
                let i1 = self.oti(object1);
                let i2 = self.oti(object2);
                match (i1, i2) {
                    (Ok(i1), Ok(i2)) => Object::Integer(i1 / i2),
                    (Err(err), _) => err,
                    (_, Err(err)) => err,
                }
            },
            Infix::Multiply => {
                let i1 = self.oti(object1);
                let i2 = self.oti(object2);
                match (i1, i2) {
                    (Ok(i1), Ok(i2)) => Object::Integer(i1 * i2),
                    (Err(err), _) => err,
                    (_, Err(err)) => err,
                }
            },
            Infix::Equal => {
                Object::Boolean(object1 == object2)
            },
            Infix::NotEqual => {
                Object::Boolean(object1 != object2)
            },
            Infix::GreaterThanEqual => {
                let i1 = self.oti(object1);
                let i2 = self.oti(object2);
                match (i1, i2) {
                    (Ok(i1), Ok(i2)) => Object::Boolean(i1 >= i2),
                    (Err(err), _) => err,
                    (_, Err(err)) => err,
                }
            },
            Infix::GreaterThan => {
                let i1 = self.oti(object1);
                let i2 = self.oti(object2);
                match (i1, i2) {
                    (Ok(i1), Ok(i2)) => Object::Boolean(i1 > i2),
                    (Err(err), _) => err,
                    (_, Err(err)) => err,
                }
            },
            Infix::LessThanEqual => {
                let i1 = self.oti(object1);
                let i2 = self.oti(object2);
                match (i1, i2) {
                    (Ok(i1), Ok(i2)) => Object::Boolean(i1 <= i2),
                    (Err(err), _) => err,
                    (_, Err(err)) => err,
                }
            },
            Infix::LessThan => {
                let i1 = self.oti(object1);
                let i2 = self.oti(object2);
                match (i1, i2) {
                    (Ok(i1), Ok(i2)) => Object::Boolean(i1 < i2),
                    (Err(err), _) => err,
                    (_, Err(err)) => err,
                }
            },
        }
    }

    pub fn eval_if(&mut self, cond: Expr, conse: BlockStmt, maybe_alter: Option<BlockStmt>) -> Object {
        let object = self.eval_expr(cond);
        match self.otb(object) {
            Ok(b) => {
                if b {
                    self.eval_blockstmt(conse)
                } else {
                    match maybe_alter {
                        Some(else_conse) => self.eval_blockstmt(else_conse),
                        _ => Object::Null,
                    }
                }
            },
            Err(err) => err,
        }
    }

    pub fn eval_fn(&mut self, params: Vec<Ident>, body: BlockStmt) -> Object {
        Object::Function(params, body, self.env.clone())
    }

    pub fn eval_call(&mut self, fn_expr: Expr, args_expr: Vec<Expr>) -> Object {
        let fn_object = self.eval_expr(fn_expr);
        let fn_ = self.otf(fn_object);
        match fn_ {
            Object::Function(params, body, f_env) =>
                self.eval_fn_call(args_expr, params, body, f_env),
            Object::Builtin(_, num_params, b_fn) =>
                self.eval_builtin_call(args_expr, num_params, b_fn),
            o_err => o_err,
        }
    }

    fn eval_fn_call(&mut self, args_expr: Vec<Expr>, params: Vec<Ident>, body: BlockStmt, f_env: Environment) -> Object {
        if args_expr.len() != params.len() {
            Object::Error(format!("wrong number of arguments: {} expected but {} given",
                params.len(),
                args_expr.len()
            ))
        } else {
            let args = args_expr
                .iter()
                .map(|&ref e| self.eval_expr(e.clone()))
                .collect::<Vec<_>>();
            let old_env = self.env.clone();
            let mut new_env = Environment::new_with_outer(box f_env);
            let zipped = params.iter().zip(args.iter());
            for (_, (ident, o)) in zipped.enumerate() {
                let Ident(name) = ident.clone();
                new_env.set(&name.clone(), o);
            }
            self.env = new_env;
            let object = self.eval_blockstmt(body);
            self.env = old_env;
            object
        }
    }

    fn eval_builtin_call(&mut self, args_expr: Vec<Expr>, num_params: usize, b_fn: BuiltinFunction) -> Object {
        if args_expr.len() != num_params {
            Object::Error(format!("wrong number of arguments: {} expected but {} given",
                num_params,
                args_expr.len()
            ))
        } else {
            Object::Null
        }
    }

    pub fn eval_array(&mut self, exprs: Vec<Expr>) -> Object {
        let new_vec = exprs
            .iter()
            .map(|&ref e| self.eval_expr(e.clone()))
            .collect::<Vec<_>>();
        Object::Array(new_vec)
    }

    pub fn object_add(&mut self, object1: Object, object2: Object) -> Object {
        match (object1, object2) {
            (Object::Integer(i1), Object::Integer(i2)) => Object::Integer(i1 + i2),
            (Object::String(s1), Object::String(s2)) => Object::String(s1 + &s2),
            (Object::Error(s), _) => Object::Error(s),
            (_, Object::Error(s)) => Object::Error(s),
            (x, y) => Object::Error(format!("{} and {} are not addable", x, y)),
        }
    }

    pub fn eval_hash(&mut self, hs: Vec<(Literal, Expr)>) -> Object {
        let mut hashmap = HashMap::new();
        for pair in hs.iter() {
            let (k, v) = self.eval_pair(pair.clone());
            hashmap.insert(k, v);
        }
        Object::Hash(hashmap)
    }

    fn eval_pair(&mut self, tuple: (Literal, Expr)) -> (Object, Object) {
        let (l, e) = tuple;
        let hash = self.l2h(l);
        let object = self.eval_expr(e);
        (hash, object)
    }

    pub fn eval_index(&mut self, target_exp: Expr, id_exp: Expr) -> Object {
        let target = self.eval_expr(target_exp);
        let index = self.eval_expr(id_exp);
        match target {
            Object::Array(arr) => {
                match self.oti(index) {
                    Ok(index_number) => {
                        let null_object = Object::Null;
                        let &ref object = arr.get(index_number as usize).unwrap_or(&null_object);
                        object.clone()
                    },
                    Err(err) => err,
                }
            },
            Object::Hash(hash) => {
                let name = self.oth(index);
                let null_object = Object::Null;
                let &ref object = hash.get(&name).unwrap_or(&null_object);
                object.clone()
            },
            o => Object::Error(format!("unexpected index target: {}", o)),
        }
    }

    pub fn otb(&mut self, object: Object) -> Result<bool, Object> {
        match object {
            Object::Boolean(b) => Ok(b),
            Object::Error(s) => Err(Object::Error(s)),
            b => Err(Object::Error(format!("{} is not a bool", b))),
        }
    }

    pub fn oti(&mut self, object: Object) -> Result<i64, Object> {
        match object {
            Object::Integer(i) => Ok(i),
            Object::Error(s) => Err(Object::Error(s)),
            i => Err(Object::Error(format!("{} is not an integer", i))),
        }
    }

    pub fn otf(&mut self, object: Object) -> Object {
        match object {
            Object::Function(_, _, _) => object,
            Object::Builtin(_, _, _) => object,
            Object::Error(s) => Object::Error(s),
            f => Object::Error(format!("{} is not a valid function", f)),
        }
    }

    pub fn oth(&mut self, object: Object) -> Object {
        match object {
            Object::Integer(i) => Object::Integer(i),
            Object::Boolean(b) => Object::Boolean(b),
            Object::String(s) => Object::String(s),
            Object::Error(s) => Object::Error(s),
            x => Object::Error(format!("{} is not hashable", x)),
        }
    }

    pub fn l2h(&mut self, literal: Literal) -> Object {
        let object = self.eval_literal(literal);
        self.oth(object)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use lexer::token::*;
    use lexer::*;
    use parser::ast::*;
    use parser::*;

    fn compare(input: &[u8], object: Object) {
        let r = Lexer::lex_tokens(input).to_result().unwrap();
        let tokens = Tokens::new(&r);
        let result_parse = Parser::parse_tokens(tokens).to_result().unwrap();
        let mut evaluator = Evaluator::new();
        let eval = evaluator.eval_program(result_parse);
        assert_eq!(eval, object);
    }

    #[test]
    fn test_simple() {
        // ints
        compare("5".as_bytes(), Object::Integer(5));
        compare("10".as_bytes(), Object::Integer(10));
        // bools
        compare("true".as_bytes(), Object::Boolean(true));
        compare("false".as_bytes(), Object::Boolean(false));
    }

    #[test]
    fn test_prefix() {
        // bang operator
        compare("!false".as_bytes(), Object::Boolean(true));
        compare("!true".as_bytes(), Object::Boolean(false));
        compare("!!false".as_bytes(), Object::Boolean(false));
        compare("!!true".as_bytes(), Object::Boolean(true));

        compare("!5".as_bytes(), Object::Error(format!("5 is not a bool")));
        compare("!1".as_bytes(), Object::Error(format!("1 is not a bool")));
        compare("!0".as_bytes(), Object::Error(format!("0 is not a bool")));
        compare("!!1".as_bytes(), Object::Error(format!("1 is not a bool")));
        compare("!!0".as_bytes(), Object::Error(format!("0 is not a bool")));
        // the prefix +
        compare("+1".as_bytes(), Object::Integer(1));
        compare("+5".as_bytes(), Object::Integer(5));
        compare("+20".as_bytes(), Object::Integer(20));
        compare("+true".as_bytes(), Object::Error(format!("true is not an integer")));
        compare("+false".as_bytes(), Object::Error(format!("false is not an integer")));
        // the prefix -
        compare("-1".as_bytes(), Object::Integer(-1));
        compare("-5".as_bytes(), Object::Integer(-5));
        compare("-20".as_bytes(), Object::Integer(-20));
        compare("-true".as_bytes(), Object::Error(format!("true is not an integer")));
        compare("-false".as_bytes(), Object::Error(format!("false is not an integer")));
    }

    #[test]
    fn test_infix_op() {
        // algebra
        compare("5 + 5 + 5 + 5 - 10".as_bytes(), Object::Integer(10));
        compare("2 * 2 * 2 * 2 * 2".as_bytes(), Object::Integer(32));
        compare("-50 + 100 + -50".as_bytes(), Object::Integer(0));
        compare("5 * 2 + 10".as_bytes(), Object::Integer(20));
        compare("5 + 2 * 10".as_bytes(), Object::Integer(25));
        compare("20 + 2 * -10".as_bytes(), Object::Integer(0));
        compare("50 / 2 * 2 + 10".as_bytes(), Object::Integer(60));
        compare("2 * (5 + 10)".as_bytes(), Object::Integer(30));
        compare("3 * 3 * 3 + 10".as_bytes(), Object::Integer(37));
        compare("3 * (3 * 3) + 10".as_bytes(), Object::Integer(37));
        compare("(5 + 10 * 2 + 15 / 3) * 2 + -10".as_bytes(), Object::Integer(50));
        // logic algebra
        compare("1 < 2".as_bytes(), Object::Boolean(true));
        compare("1 > 2".as_bytes(), Object::Boolean(false));
        compare("1 < 1".as_bytes(), Object::Boolean(false));
        compare("1 > 1".as_bytes(), Object::Boolean(false));
        compare("1 <= 2".as_bytes(), Object::Boolean(true));
        compare("1 >= 2".as_bytes(), Object::Boolean(false));
        compare("1 <= 1".as_bytes(), Object::Boolean(true));
        compare("1 >= 1".as_bytes(), Object::Boolean(true));
        compare("1 == 1".as_bytes(), Object::Boolean(true));
        compare("1 != 1".as_bytes(), Object::Boolean(false));
        compare("1 == 2".as_bytes(), Object::Boolean(false));
        compare("1 != 2".as_bytes(), Object::Boolean(true));
        // combination
        compare("(1 < 2) == true".as_bytes(), Object::Boolean(true));
        compare("(1 < 2) == false".as_bytes(), Object::Boolean(false));
        compare("(1 > 2) == true".as_bytes(), Object::Boolean(false));
        compare("(1 > 2) == false".as_bytes(), Object::Boolean(true));
    }

    #[test]
    fn test_conditional() {
        compare("if (true) { 10 }".as_bytes(), Object::Integer(10));
        compare("if (false) { 10 }".as_bytes(), Object::Null);
        compare("if (1) { 10 }".as_bytes(), Object::Error(format!("1 is not a bool")));
        compare("if (1 < 2) { 10 }".as_bytes(), Object::Integer(10));
        compare("if (1 > 2) { 10 }".as_bytes(), Object::Null);
        compare("if (1 < 2) { 10 } else { 20 }".as_bytes(), Object::Integer(10));
        compare("if (1 > 2) { 10 } else { 20 }".as_bytes(), Object::Integer(20));
    }

    #[test]
    fn test_return() {
        compare("return 10".as_bytes(), Object::Integer(10));
        compare("return 10; 9".as_bytes(), Object::Integer(10));
        compare("return 2 * 5; 9".as_bytes(), Object::Integer(10));
        compare("9; return 2 * 5; 9".as_bytes(), Object::Integer(10));

        let input =
            "if (10 > 1) {\
                 if (10 > 1) {\
                     return 10;\
                 }\
                 return 1;\
             }\
            ".as_bytes();
        compare(input, Object::Integer(10));
    }

    #[test]
    fn test_bindings() {
        compare("let a = 5; a;".as_bytes(), Object::Integer(5));
        compare("let a = 5 * 5; a;".as_bytes(), Object::Integer(25));
        compare("let a = 5; let b = a; b;".as_bytes(), Object::Integer(5));
        compare("let a = 5; let b = a; let c = a + b + 5; c;".as_bytes(), Object::Integer(15));
        compare("foobar".as_bytes(), Object::Error(format!("identifier not found: foobar")));
    }

    #[test]
    fn test_strings() {
        compare("\"foobar\"".as_bytes(), Object::String("foobar".to_string()));
        compare("\"foo\" + \"bar\"".as_bytes(), Object::String("foobar".to_string()));
        compare("\"foo\" + \" \" + \"bar\"".as_bytes(), Object::String("foo bar".to_string()));
        compare("\"foo\" - \"bar\"".as_bytes(), Object::Error(format!("foo is not an integer")));
    }

    #[test]
    fn test_fn() {
        compare("let identity = fn(x) { x; }; identity(5);".as_bytes(), Object::Integer(5));
        compare("let identity = fn(x) { return x; }; identity(5);".as_bytes(), Object::Integer(5));
        compare("let double = fn(x) { x * 2; }; double(5);".as_bytes(), Object::Integer(10));
        compare("let add = fn(x, y) { x + y; }; add(5, 5);".as_bytes(), Object::Integer(10));
        compare("let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));".as_bytes(), Object::Integer(20));
        compare("fn(x) { x; }(5)".as_bytes(), Object::Integer(5));
        compare("5();".as_bytes(), Object::Error(format!("5 is not a valid function")));
        compare("false();".as_bytes(), Object::Error(format!("false is not a valid function")));
        compare("let add = fn(x, y) { x + y; }; add(1);".as_bytes(), Object::Error(format!(
            "wrong number of arguments: 2 expected but 1 given"
        )));
        compare("let a = 10; let x = fn () { a; }; x();".as_bytes(), Object::Integer(10));
        // todo
        // compare("let x = fn () { a; }; let a = 10; x();".as_bytes(), Object::Integer(10));
    }

    #[test]
    fn test_array() {
        // todo let double = fn(x) { x * 2 };[1, double(2), 3 * 3, 4 - 3]
        compare("[1, 2, 3, 4]".as_bytes(), Object::Array(vec!(
            Object::Integer(1),
            Object::Integer(2),
            Object::Integer(3),
            Object::Integer(4),
        )));

        compare("let double = fn(x) { x * 2 };[1, double(2), 3 * 3, 4 - 3]".as_bytes(),
            Object::Array(vec!(
                Object::Integer(1),
                Object::Integer(4),
                Object::Integer(9),
                Object::Integer(1),
            )
        ));

        compare("[1, 2, 3][0]".as_bytes(), Object::Integer(1));
        compare("[1, 2, 3][1]".as_bytes(), Object::Integer(2));
        compare("[1, 2, 3][2]".as_bytes(), Object::Integer(3));
        compare("let i = 0; [1][i];".as_bytes(), Object::Integer(1));
        compare("[1, 2, 3][1 + 1];".as_bytes(), Object::Integer(3));
        compare("let myArray = [1, 2, 3]; myArray[2];".as_bytes(), Object::Integer(3));
        compare("let myArray = [1, 2, 3]; myArray[0] + myArray[1] + myArray[2];".as_bytes(), Object::Integer(6));
        compare("let myArray = [1, 2, 3]; let i = myArray[0]; myArray[i];".as_bytes(), Object::Integer(2));
        compare("[1, 2, 3][3]".as_bytes(), Object::Null);
        compare("[1, 2, 3][-1]".as_bytes(), Object::Null);
    }

    #[test]
    fn test_hash() {
        let input_beg =
        "let double = fn(x) {
           x * 2;
         };
         let arr = [1, 2, 3, 4];
         let h = {
           \"one\": 10 - 9,
           \"two\": 8 / 4,
           3: arr[2],
           4: double(2),
           true: if (10 > 8) { true } else { false },
           false: \"hello\" == \"world\"
         };
        ".to_string();

        compare((input_beg.clone() + &"h[\"one\"]".to_string()).as_bytes(), Object::Integer(1));
        compare((input_beg.clone() + &"let s = \"two\"; h[s]".to_string()).as_bytes(), Object::Integer(2));
        compare((input_beg.clone() + &"h[3]".to_string()).as_bytes(), Object::Integer(3));
        compare((input_beg.clone() + &"h[2 + 2]".to_string()).as_bytes(), Object::Integer(4));
        compare((input_beg.clone() + &"h[true]".to_string()).as_bytes(), Object::Boolean(true));
        compare((input_beg.clone() + &"h[5 < 1]".to_string()).as_bytes(), Object::Boolean(false));
        compare((input_beg.clone() + &"h[100]".to_string()).as_bytes(), Object::Null);
        compare((input_beg.clone() + &"h[[]]".to_string()).as_bytes(), Object::Null);
        compare((input_beg.clone() + &"3[true];".to_string()).as_bytes(),
            Object::Error(format!("unexpected index target: 3")));
    }
}