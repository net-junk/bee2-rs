
fn check(data: impl AsRef<[u8]>)
{
    let data = data.as_ref();
    println!("{:?}", data.len())
}

fn main() {
    println!("Hello, world!");
    let array :[u8; 5] = [0, 0, 1, 2, 3];
    check(array);
}
