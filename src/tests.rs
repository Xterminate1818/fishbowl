#[cfg(test)]
#[test]
fn bench_async() {
  use std::io::Write;
  flexi_logger::Logger::try_with_str("debug").unwrap();

  let image = image::io::Reader::open(
    "resources/mona
  lisa.png",
  )
  .unwrap()
  .decode()
  .unwrap()
  .to_rgb8();
  // let gif = pollster::block_on(crate::generate_async(image, None));
  // let mut file =
  // std::fs::File::create("test.gif").unwrap();
  // file.write(&gif).unwrap();
}
