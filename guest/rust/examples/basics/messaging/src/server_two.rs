use ambient_api::{
    message::server::{MessageExt, Source, Target},
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    messages::Local::subscribe(move |source, data| {
        println!("{source:?}: {data:?}");
        if let Source::Module(id) = source {
            messages::Local {
                text: "Hi, back!".into(),
            }
            .send(Target::Module(id));
        }

        EventOk
    });

    EventOk
}
