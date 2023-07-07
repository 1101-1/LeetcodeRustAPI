use std::time::Duration;

use serde_json::json;

use crate::{resources::{
    problemfulldata::{
        CodeSnippetNode, ProblemFullData, SimilarQuestions, Solution, Statistics, TopicTagNode,
    },
    subm_send::{SubmExecutionResult, SubmissionCase, SubmissionCaseResp},
    subm_show::SubmList,
    test_send::{TestCase, TestCaseResp, TestExecutionResult},
    Descryption, Rate,
}, error::Errors};

#[derive(Debug)]
pub struct Problem {
    pub client: reqwest::Client,
    pub task_search_name: String,
    pub full_data: ProblemFullData,
}

#[allow(unused)]
impl Problem {
    pub async fn send_test(
        &self,
        lang: &str,
        typed_code: &str,
    ) -> Result<TestExecutionResult, Errors> {
        let json_data = serde_json::to_string(&TestCase {
            question_id: self.full_data.data.question.questionId.clone(),
            data_input: self.full_data.data.question.sampleTestCase.clone(),
            lang: lang.to_lowercase(),
            judge_type: String::from("large"),
            typed_code: String::from(typed_code),
        })?;

        let resp = self
            .client
            .post(format!(
                "https://leetcode.com/problems/{}/interpret_solution/",
                self.task_search_name
            ))
            .body(json_data)
            .send()
            .await?
            .json::<TestCaseResp>()
            .await?;

        loop {
            let status = self
                .client
                .get(format!(
                    "https://leetcode.com/submissions/detail/{}/check/",
                    resp.interpret_id
                ))
                .send()
                .await?
                .json::<TestExecutionResult>()
                .await?;
            if status.state == "SUCCESS" {
                return Ok(status);
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn send_subm(
        &self,
        lang: &str,
        code: &str,
    ) -> Result<SubmExecutionResult, Errors> {
        let json_data = serde_json::to_string(&SubmissionCase {
            question_id: self.full_data.data.question.questionId.clone(),
            lang: lang.to_lowercase(),
            typed_code: String::from(code),
        })
        .unwrap();

        let resp = self
            .client
            .post(format!(
                "https://leetcode.com/problems/{}/submit/",
                self.task_search_name
            ))
            .body(json_data)
            .send()
            .await?
            .json::<SubmissionCaseResp>()
            .await?;

        loop {
            let status = self
                .client
                .get(format!(
                    "https://leetcode.com/submissions/detail/{}/check/",
                    resp.submission_id
                ))
                .send()
                .await?
                .json::<SubmExecutionResult>()
                .await?;
            if status.state == "SUCCESS" {
                return Ok(status);
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
    pub fn code_snippets(&self) -> Option<Vec<CodeSnippetNode>> {
        self.full_data.data.question.codeSnippets.clone()
    }

    pub fn solution_info(&self) -> Option<Solution> {
        self.full_data.data.question.solution.clone()
    }

    pub fn related_topics(&self) -> Vec<TopicTagNode> {
        self.full_data.data.question.topicTags.clone()
    }

    pub fn similar_questions(&self) -> Vec<SimilarQuestions> {
        serde_json::from_str::<Vec<SimilarQuestions>>(
            self.full_data.data.question.similarQuestions.as_str(),
        )
        .unwrap()
    }
    pub fn stats(&self) -> Statistics {
        serde_json::from_str::<Statistics>(self.full_data.data.question.stats.as_str()).unwrap()
    }

    pub fn hints(&self) -> Vec<String> {
        self.full_data.data.question.hints.clone()
    }

    pub fn description(&self) -> Descryption {
        let descryption = json!({
            "name": self.full_data.data.question.title,
            "content": self.full_data.data.question.content
        });
        serde_json::from_value::<Descryption>(descryption).unwrap()
    }

    pub fn difficulty(&self) -> String {
        self.full_data.data.question.difficulty.clone()
    }

    pub fn rating(&self) -> Rate {
        let rate = json!({
            "likes": self.full_data.data.question.likes,
            "dislikes": self.full_data.data.question.dislikes
        });

        serde_json::from_value::<Rate>(rate).unwrap()
    }

    pub fn category(&self) -> String {
        self.full_data.data.question.categoryTitle.clone()
    }

    pub async fn my_submissions(&self) -> Result<SubmList, Errors> {
        let query = json!({
            "operationName": "Submissions",
            "variables": {
                "offset": 0,
                "limit": 20,
                "lastKey": null,
                "questionSlug": self.task_search_name
            },
            "query": "query Submissions($offset: Int!, $limit: Int!, $lastKey: String, $questionSlug: String!) {\n  submissionList(offset: $offset, limit: $limit, lastKey: $lastKey, questionSlug: $questionSlug) {\n    lastKey\n    hasNext\n    submissions {\n      id\n      statusDisplay\n      lang\n      runtime\n      timestamp\n      url\n      isPending\n      memory\n      __typename\n    }\n    __typename\n  }\n}\n"
        });

        let query = serde_json::to_string(&query)?;

        Ok(self
            .client
            .post("https://leetcode.com/graphql/")
            .body(query)
            .send()
            .await?
            .json::<SubmList>()
            .await?)
    }
}
