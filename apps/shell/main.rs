use boson::id::Id;
use boson::sample::add;

fn main() {
    let id = Id::zero();
    println!("{}", id);

    println!("{}", add(1, 2));
}
