use chargen::CharacterGenerator;
// use std::collections::HashMap;

fn main() {
    let generator = CharacterGenerator::from_files("../characters", "../cjkvi-ids/ids.txt").unwrap();
 
    /*
    let unavailable = generator.mappings.iter()
        .filter(|(_, (available, _))| *available)
        .filter(|(_, (_, mapping))| mapping.chars().count() > 1)
        .map(|(ch, _)| *ch)
        .collect::<Vec<char>>();

    println!("{}", unavailable.len());
    println!("{}", generator.mappings.iter().count());
    println!("{:?}", unavailable);
    */

    // println!("綈 {:?}", generator.get_mapping('綈').unwrap());

    /*
    let mut counts:HashMap<char, usize> = HashMap::new();
    for mapping in generator.mappings.iter()
        .filter(|(_, (_, mapping))| mapping.chars().count() > 1)
        .map(|(_, (_,  mapping))| mapping) {
        let ch = mapping.chars().next().unwrap();
        let count = counts.entry(ch).or_insert(0);
        *count += 1;
    }

    println!("{:?}", counts);
    */

    // println!("腰 {:?}", generator.get_mapping('腰').unwrap());
    // println!("{}", generator.svg('月').unwrap());
    // println!("{}", generator.svg('要').unwrap());
    
    println!("{}", generator.svg('腰').unwrap());
}
