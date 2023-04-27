use axum::response::Html;

pub async fn show_status() -> Html<&'static str> {
    Html(
        r#"
<html>
    <head>
        <title>Ussal Runner Status</title>
        <style>
            body {
                color: rgb(255, 255, 255);
                background-color: rgb(30, 30, 30) !important;
            }
        </style>
    </head>

    <body>
        <h1>Job Queue</h1>
        <h1>Cluster State</h1>
    </body>
</html>
"#,
    )
}
