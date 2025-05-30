use emojis::emoji::Emoji;
use emojis::emoji::Group;
pub fn get_emoji_section() -> Vec<&'static Emoji> {
    Group::SmileysAndEmotion.emojis().collect()
}
