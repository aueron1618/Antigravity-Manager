pub const FAKE_STREAM_PREFIX: &str = "\u{5047}\u{6d41}\u{5f0f}/";

pub fn strip_fake_stream_prefix(model: &str) -> (String, bool) {
    if let Some(stripped) = model.strip_prefix(FAKE_STREAM_PREFIX) {
        return (stripped.to_string(), true);
    }
    (model.to_string(), false)
}

pub fn is_image_model(model: &str) -> bool {
    model.to_lowercase().contains("image")
}

pub fn append_fake_stream_prefixes(models: Vec<String>, enabled: bool) -> Vec<String> {
    if !enabled {
        return models;
    }

    let mut all_models = models;
    let prefixed: Vec<String> = all_models
        .iter()
        .filter(|model| !is_image_model(model))
        .map(|model| format!("{}{}", FAKE_STREAM_PREFIX, model))
        .collect();

    all_models.extend(prefixed);
    all_models
}
