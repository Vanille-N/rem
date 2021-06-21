trait Bar {}

struct Foo<'s> { data: Box<dyn Bar + 's> }
struct Baz(u64);
impl Bar for Baz {}

impl<'s> Foo<'s> { fn new<B>(b: B) -> Self where B: Bar + 's { Self { data: Box::new(b) } } }

fn main() {
    let f = Foo::new(Baz(0));
}
