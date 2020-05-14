use super::visitor::*;
use zub::ir::*;
use zub::vm::*;

pub fn include_math(visitor: &mut Visitor, vm: &mut VM) {
    visitor.set_global("sum", TypeNode::Func(1));
    vm.add_native("sum", sum, 1);
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