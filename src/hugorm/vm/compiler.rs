use super::super::parser::*;
use super::*;

pub fn compile(ast: &Vec<Statement>) -> Vec<Op> {
    let mut program = Vec::new();

    use self::StatementNode::*;

    for statement in ast.iter() {
        match statement.node {
            Declaration(ref left, ref right) => {
                if let Some(right) = right {
                    compile_expr(&right, &mut program)
                } else {
                    unimplemented!() // succ ma dicc
                }

                program.push(Op::PushLocal)
            }

            Expression(ref expr) => compile_expr(expr, &mut program),

            _ => ()
        }
    }

    program.push(Op::Ret);

    program
}

fn compile_expr(expr: &Expression, program: &mut Vec<Op>) {
    use self::ExpressionNode::*;
    
    match expr.node {
        Int(ref x)   => program.push(Op::Int(*x)),
        Float(ref x) => program.push(Op::Float(*x)),
        Identifier(ref id) => program.push(Op::Local(0)),
        _ => ()
    }
}