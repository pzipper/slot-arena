use slot_arena::SlotArena;

fn main() {
    let mut names: SlotArena<&'_ str> = SlotArena::new();

    {
        let _value1 = names.insert("James");
        let value2 = names.insert("John");
        let _value3 = names.insert("Jack");
        names.free(value2);
    }

    dbg!(names);
}
