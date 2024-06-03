use fantoccini::{Client, Locator};
use serde::{Deserialize, Serialize};
use std::{fs, time::Duration};
use tokio;

#[derive(Deserialize, Serialize, Debug)]
pub struct TestStep {
    visit: Option<String>,
    find: Option<String>,
    #[serde(default)]
    type_text: Option<String>,
    #[serde(default)]
    click: Option<bool>,
    #[serde(default)]
    wait: Option<u64>,
    assert: Option<Vec<Assertion>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Assertion {
    array: Option<String>,
    empty: Option<String>,
    string: Option<String>,
    equal: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestCase {
    name: String,
    description: String,
    steps: Vec<TestStep>,
}

pub async fn run_browser_tests(
    test_cases: &Vec<TestCase>,
) -> Result<(), fantoccini::error::CmdError> {
    println!("Running browser tests...");

    let mut client = Client::new("http://localhost:4444")
        .await
        .expect("Failed to connect to WebDriver");

    for test_case in test_cases {
        println!("Executing test case: {}", test_case.name);
        for step in &test_case.steps {
            if let Some(url) = &step.visit {
                client.goto(url).await?;
            }

            if let Some(selector) = &step.find {
                let element = client.find(Locator::Css(selector)).await?;
                if let Some(text) = &step.type_text {
                    element.send_keys(text).await?;
                }
                if step.click.unwrap_or(false) {
                    element.click().await?;
                }
            }

            if let Some(duration) = step.wait {
                tokio::time::sleep(Duration::from_millis(duration)).await;
            }

            if let Some(assertions) = &step.assert {
                for assertion in assertions {
                    if let Some(selector) = &assertion.array {
                        let elements = client.find_all(Locator::Css(selector)).await?;
                        assert!(!elements.is_empty(), "Expected array but found none");
                    }

                    if let Some(selector) = &assertion.empty {
                        let elements = client.find_all(Locator::Css(selector)).await?;
                        assert!(elements.is_empty(), "Expected no elements but found some");
                    }

                    if let Some(selector) = &assertion.string {
                        let element = client.find(Locator::Css(selector)).await?;
                        let text = element.text().await?;
                        assert!(
                            text.parse::<String>().is_ok(),
                            "Expected string but found something else"
                        );
                    }

                    if let Some(equal) = &assertion.equal {
                        let parts: Vec<&str> = equal.split("==").collect();
                        if parts.len() == 2 {
                            let selector = parts[0].trim();
                            let expected_value = parts[1].trim().trim_matches('"');
                            let element = client.find(Locator::Css(selector)).await?;
                            let text = element.text().await?;
                            assert_eq!(
                                text, expected_value,
                                "Expected '{}' but found '{}'",
                                expected_value, text
                            );
                        }
                    }
                }
            }
        }
    }

    client.close().await?;
    Ok(())
}
