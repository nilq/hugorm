use broom::prelude::*;

make_nanbox! {
    #[derive(Clone, Debug, PartialEq)]
    pub unsafe enum Value, Variant {
        Float(f64),
        Int(i32),
        Bool(u8),
        Pointer(u32)
    }
}

impl Value {
    #[inline]
    pub fn as_heap(self) -> HeapObject {
        HeapObject::Value(self)
    }
}

#[derive(Debug)]
pub enum HeapObject {
    Value(Value),
    Str(String),
    List(Vec<Handle<Self>>),
}

impl Trace<Self> for HeapObject {
    fn trace(&self, tracer: &mut Tracer<Self>) {
        use self::HeapObject::*;

        match self {
            List(objects) => objects.trace(tracer),
            _ => {}
        }
    }
}