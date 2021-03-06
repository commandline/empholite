use super::{types::Mode, Editor, Msg};
use crate::{components::alert::Context, Rule};
use anyhow::{format_err, Context as _, Result};
use log::error;
use std::convert::TryInto;
use validator::Validate;
use yew::{
    format::{Nothing, Text},
    prelude::*,
    services::{
        fetch::{Request, Response, StatusCode},
        FetchService,
    },
};

impl Editor {
    pub(super) fn handle_edit(&mut self) -> Result<ShouldRender> {
        self.mode = Mode::Edit;
        Ok(true)
    }

    pub(super) fn handle_cancel(&mut self) -> Result<ShouldRender> {
        self.mode = Mode::View;
        self.errors = None;
        self.link.send_message(Msg::Fetch);
        Ok(false)
    }

    pub(super) fn handle_fetch(&mut self) -> Result<ShouldRender> {
        let request = Request::get(format!(
            "/ajax/recipe/{}",
            self.props
                .id
                .ok_or_else(|| format_err!("Cannot fetch recipe, ID is not set!"))?
        ))
        .body(Nothing)
        .map_err(anyhow::Error::from)?;
        let task = FetchService::fetch(
            request,
            self.link.callback(
                move |response: Response<Text>| match response.into_parts() {
                    (meta, Ok(body)) if meta.status >= StatusCode::BAD_REQUEST => {
                        Msg::Failure(body)
                    }
                    (_, Ok(body)) => Msg::Fetched(body),
                    (_, Err(error)) => {
                        error!("{}", error);
                        Msg::Failure(format!("{}", error))
                    }
                },
            ),
        )?;
        self.fetch_tsk = Some(task);
        Ok(false)
    }

    pub(super) fn handle_fetched(&mut self, body: String) -> Result<ShouldRender> {
        let state: shared::Recipe = serde_json::from_str(&body)
            .with_context(|| "Error parsing JSON when trying to fetch a recipe!")?;
        self.state = state.into();
        self.fetch_tsk = None;
        self.link.send_message(Msg::FetchConfig);
        Ok(true)
    }

    pub(super) fn handle_fetch_config(&mut self) -> Result<ShouldRender> {
        let request = Request::get("/ajax/config")
            .body(Nothing)
            .map_err(anyhow::Error::from)?;
        let task = FetchService::fetch(
            request,
            self.link.callback(
                move |response: Response<Text>| match response.into_parts() {
                    (meta, Ok(body)) if meta.status >= StatusCode::BAD_REQUEST => {
                        Msg::Failure(body)
                    }
                    (_, Ok(body)) => Msg::FetchedConfig(body),
                    (_, Err(error)) => {
                        error!("{}", error);
                        Msg::Failure(format!("{}", error))
                    }
                },
            ),
        )?;
        self.fetch_tsk = Some(task);
        Ok(false)
    }

    pub(super) fn handle_fetched_config(&mut self, body: String) -> Result<ShouldRender> {
        let config: shared::Config = serde_json::from_str(&body)
            .with_context(|| "Error parsing JSON when trying to fetch config!")?;
        self.fetch_tsk = None;
        self.config = config;
        Ok(true)
    }

    pub(super) fn handle_url_change(&mut self, url: String) -> Result<ShouldRender> {
        self.state.url = url;
        Ok(true)
    }

    pub(super) fn handle_payload_change(&mut self, payload: String) -> Result<ShouldRender> {
        self.state.payload = payload;
        Ok(true)
    }

    pub(super) fn handle_failure(&mut self, error: String) -> Result<ShouldRender> {
        self.alert_ctx = Context::Danger(error);
        Ok(true)
    }

    pub(super) fn handle_add_rule(&mut self) -> Result<ShouldRender> {
        self.state.rules.push(Rule::default());
        Ok(true)
    }

    pub(super) fn handle_rule_changed(&mut self, rule: Rule, index: usize) -> Result<ShouldRender> {
        self.state.rules[index] = rule;
        Ok(true)
    }

    pub(super) fn handle_remove_rule(&mut self, index: usize) -> Result<ShouldRender> {
        self.state.rules.remove(index);
        Ok(true)
    }

    pub(super) fn handle_post(&mut self) -> Result<ShouldRender> {
        if let Err(errors) = self.state.validate() {
            error!("Validation errors {:?}", errors);
            self.link.send_message(Msg::Failure(
                "There were problems with this recipe. Please fix the errors, below, and try saving
                again."
                    .into(),
            ));
            self.errors = Some(errors);
            Ok(true)
        } else {
            let body: shared::Recipe = self.state.clone().try_into()?;
            let request = Request::post("/ajax/recipe/")
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&body).map_err(anyhow::Error::from))
                .map_err(anyhow::Error::from)?;
            let task = FetchService::fetch(
                request,
                self.link.callback(
                    move |response: Response<Text>| match response.into_parts() {
                        (meta, Ok(body)) if meta.status >= StatusCode::BAD_REQUEST => {
                            Msg::Failure(body)
                        }
                        (_, Ok(body)) => Msg::Posted(body),
                        (_, Err(error)) => {
                            error!("{}", error);
                            Msg::Failure(format!("{}", error))
                        }
                    },
                ),
            )?;
            self.fetch_tsk = Some(task);
            self.errors = None;
            Ok(false)
        }
    }

    pub(super) fn handle_posted(&mut self, body: String) -> Result<ShouldRender> {
        let state: shared::Recipe = serde_json::from_str(&body)?;
        self.state = state.into();
        self.alert_ctx = Context::Success("Saved!".into());
        self.mode = Mode::View;
        self.fetch_tsk = None;
        Ok(true)
    }
}
