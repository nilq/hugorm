use super::visitor::*;
use zub::ir::*;
use zub::vm::*;

use statrs::distribution::StudentsT;
use statrs::statistics::*;

pub fn include_math(visitor: &mut Visitor, vm: &mut VM) {
    visitor.set_global("sum", TypeNode::Func(1));
    vm.add_native("sum", sum, 1);

    visitor.set_global("student", TypeNode::Func(3));
    vm.add_native("student", student, 3);
}

fn sum(heap: &mut Heap<Object>, args: &[Value]) -> Value {
    if let Variant::Obj(handle) = args[1].decode() {
        let list = unsafe { heap.get_unchecked(handle) };

        if let Some(list) = list.as_list() {
            let mut sum = 0f64;

            for item in list.content.iter() {
                if let Variant::Float(n) = item.decode() {
                    sum += n
                } else {
                    panic!("can't sum non-float")
                }
            }

            return Value::float(sum)
        } else {
            panic!("can't sum non-list")
        }
    }

    panic!("can't sum non-list: {:#?}", args[1])
}

fn student(heap: &mut Heap<Object>, args: &[Value]) -> Value {
    let floats = args[1..].iter().take(3).map(|x| {
            if let Variant::Float(n) = x.decode() {
                n
            } else {
                panic!("student can't take non-float")
            }
        })
        .collect::<Vec<f64>>();

    let t = StudentsT::new(floats[0], floats[1], floats[2]).unwrap();

    println!("{}", t.mean());

    Value::nil()
}