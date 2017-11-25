
error_chain! {
    // Bindings to types implementing std::error::Error.
    foreign_links {
        Io(::std::io::Error);
        UrlParse(::url::ParseError);
        Git(::git2::Error);
        Serde(::serde_json::Error);
        Hyper(::hyper::Error);
        UriError(::hyper::error::UriError);
    }
}
