#![feature(async_await)]
#![feature(await_macro)]
#![feature(futures_api)]

use moiety::*;

async fn go() -> Result<(), MhkError> {
    for_each_riven!(|r| => {
        println!("found resource type: {:?}", r);
    });
    
    //let fs = LoggingFilesystem::new("root", LocalFilesystem::new("/home/agrif/vault/games/riven/"));
    let fs = LoggingFilesystem::new("root", LocalFilesystem::new("./local/"));
    
    //let map = MhkMap::new(fs);
    let map = DirectMap::new(fs);

    //let fmt = MhkFormat;
    let fmt = JsonFormat;
    
    let rs = Resources::new_with_map_error(map, fmt);
    let resource = await!(rs.open(RivenStack::A, Riven::NAME, 2))?;
    println!("{:?}", Riven::NAME);
    println!("{:?}", resource);

    //println!("{}", serde_json::to_string_pretty(&resource).unwrap());

    Ok(())
}

fn main() -> Result<(), MhkError> {
    futures::executor::block_on(go())
}
