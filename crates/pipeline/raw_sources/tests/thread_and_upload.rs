use deer_pipeline_raw_sources::{create_thread, resume_thread, stage_upload};

#[test]
fn creates_or_resumes_threads_and_stages_uploads_before_submit() {
    let created = create_thread("Survey the ridge").unwrap();
    let resumed = resume_thread(include_str!("fixtures/thread_resume.json")).unwrap();
    let upload = stage_upload(created.thread_id.as_str(), "briefing.pdf").unwrap();

    assert_eq!(created.title, "Survey the ridge");
    assert_eq!(resumed.thread_id, "thread_1");
    assert_eq!(upload.file_name, "briefing.pdf");
}
