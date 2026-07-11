/// 尝试多种编码解析文件内容
pub fn decode_file_content(bytes: &[u8]) -> String {
    // 尝试的编码列表：UTF-8, GBK(Windows简体中文), GB18030, Latin1
    let encodings = [
        encoding_rs::UTF_8,
        encoding_rs::GBK,
        encoding_rs::GB18030,
        encoding_rs::WINDOWS_1252, // Latin1/Windows西欧
    ];

    for encoding in &encodings {
        let (cow, _, had_errors) = encoding.decode(bytes);
        if !had_errors {
            return cow.into_owned();
        }
    }

    // 如果所有编码都失败，使用 UTF-8 lossy 解码（替换非法字符为 �）
    String::from_utf8_lossy(bytes).into_owned()
}