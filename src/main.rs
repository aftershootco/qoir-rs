fn main() {
    println!("Hello, world!");

    let bytes =
        include_bytes!("C:\\Users\\Jaysmito\\Downloads\\at-mouquins.lossy-naive-dither-2.qoir");

    let decoded = qoir_rs::decode_from_memory(bytes, qoir_rs::DecodeOptions::default()).unwrap();

    println!("{:?}", decoded.image.width);
    println!("{:?}", decoded.image.height);
    println!("{:?}", decoded.image.pixel_format);
    println!("{:?}", decoded.image.stride_in_bytes);
}
