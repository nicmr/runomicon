use std::path::PathBuf;
use std::fs::read_to_string;
// use std::thread;
use std::path::Path;

use iced::{
    button, Application, Button, Column, Command,
    Container, Element, Length, Row, Settings, Text, window,
};

mod league;
use league::{Lockfile, LolPerksPerkPage, LolPerksPerkUIPerk};
use std::collections::{HashMap};


mod error;
use error::{Error, StringError};

fn main() {
    Runomicon::run(
        Settings {
            window: window::Settings::default(),
            flags: (),
            default_font: None,
            antialiasing: false,
        }
    );
}

struct Runomicon {
    screen: Screen,
    league_status: LeagueStatus,
    runes_by_id: Option<HashMap<i64, LolPerksPerkUIPerk>>,
    runepages: Option<Vec<LolPerksPerkPage>>,
}

struct LeagueStatus {
    league_path: Option<PathBuf>,
    lockfile: Option<Lockfile>,
}

impl LeagueStatus {
    fn new() -> Self {
        Self {
            league_path: None,
            lockfile: None,
        }
    }
    fn dir_ok(&self) -> bool {
        unimplemented!();
    }

    fn read_lockfile(&mut self) -> Result<(), Error> {
        // read and parse the lockfile
        // TODO: check fields with serde, parser combinator or regex instead
        if let Some(path) = &self.league_path {
            let lockfile_path = path.join("lockfile");
            let lockfile = {
                let contents = read_to_string(lockfile_path)?;
                let v: Vec<&str> = contents.split(':').collect();
                if v.len() < 5 {
                    Err(Error::Simple(StringError::new("LoL lockfile too short")))
                } else {
                    Ok(Lockfile {
                        process: v[0].to_owned(),
                        pid: v[1].parse()?,
                        port: v[2].parse()?,
                        password: v[3].to_owned(),
                        protocol: v[4].to_owned(),
                    })
                }
            }?;
            self.lockfile = Some(lockfile);
            Ok(())
        } else {
            Err(Error::Simple(StringError::new("No path provided")))
        }
    }
}

enum Screen {
    LocateLeagueDir{btn_states: LocateLeagueDirBtnStates},
    RunepageDisplay{overviews: Vec<RunepageOverview>},
    RunepageModify{id: usize},
    Normal,
}

struct LocateLeagueDirBtnStates {
    choose_dir_btn: button::State,
    go_to_runes_btn: button::State,
}

impl LocateLeagueDirBtnStates {
    pub fn new () -> Self {
        Self {
            choose_dir_btn: button::State::new(),
            go_to_runes_btn: button::State::new(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    DirLocated,
    PickFolder,
    PickFolderDone(Result<PathBuf, Error>),
    GetRunesDone(Result<String, Error>),
    GetRunepagesDone(Result<String, Error>),
    GoToRunes,
    RunepageOverviewMessage(usize, RunepageOverviewMessage)
}

impl Application for Runomicon {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Runomicon, Command<Message>) {
        (Runomicon {
            screen: Screen::LocateLeagueDir{btn_states: LocateLeagueDirBtnStates::new()},
            league_status: LeagueStatus::new(),
            runes_by_id: None,
            runepages: None,
        }
        , Command::none())
    }

    fn title(&self) -> String {
        String::from("runomicon")
    }

    fn update (&mut self, message: Message) -> Command<Message> {
        match message {
            Message::DirLocated => {
                self.screen =  match self.screen {
                    Screen::LocateLeagueDir{btn_states: _} => Screen::Normal,
                    _ => Screen::Normal,
                };
                Command::none()
            }, 
            Message::PickFolder => {
                Command::perform(Runomicon::pick_folder(None), Message::PickFolderDone)
            },
            Message::PickFolderDone(result) => {
                match result {
                    Ok(path) => {
                        self.league_status.league_path = Some(path);
                        match self.league_status.read_lockfile() {
                            Ok(()) => (),
                            Err(e) => println!("{:?}", e)
                        };
                    },
                    Err(e) => {
                        println!("{:?}", e);
                    }
                }
                Command::none()
            },
            Message::GoToRunes => {
                if let Some (lf) = &self.league_status.lockfile {
                    self.screen = Screen::RunepageDisplay {overviews: Vec::new()};
                    Command::perform(Runomicon::get_runes(lf.clone()), Message::GetRunesDone)
                } else {
                    Command::none()
                }
            },
            Message::GetRunesDone (result) => {
                match result {
                    Ok(s) => {
                        let runes: Vec<LolPerksPerkUIPerk> = serde_json::from_str(&s).unwrap();
                        let rune_map = runes
                            .iter()
                            .fold(HashMap::new(), |mut map, perk| {
                            map.insert(perk.id, perk.clone());
                            map
                        });

                        self.runes_by_id = Some(rune_map);
                    }
                    Err(e) => {
                        println!("{:?}", e);
                    }
                }
                if let Some (lf) = &self.league_status.lockfile {
                    Command::perform(Runomicon::get_runepages(lf.clone()), Message::GetRunepagesDone)
                } else {
                    Command::none()
                }
            }
            Message::GetRunepagesDone (result) => {
                match result {
                    Ok(s) => {
                        // Careful, properly handle unwrap
                        let deserialized_pages: Vec<LolPerksPerkPage> = serde_json::from_str(&s).unwrap();
                        self.runepages = Some(deserialized_pages);
                        self.screen = Screen::RunepageDisplay {
                            overviews: self.runepages.clone().iter().flatten().map(|page| RunepageOverview::from(page)).collect()
                        };
                    }
                    Err(e) => {
                        println!("{:?}", e);
                    }
                }
                Command::none()
            },
            Message::RunepageOverviewMessage (index, msg ) => {
                self.screen = Screen::RunepageModify { id: index } ;
                Command::none()
            },
        }
    }

    fn view (&mut self ) -> Element<Message> {
        let content = match &mut (self.screen) {
            Screen::LocateLeagueDir{btn_states} => {
                let displayed_path = match &self.league_status.league_path {
                    Some(path) => {
                        match path.to_str() {
                            Some(s) => s.to_owned(),
                            None => String::from("<Path contains invalid unicode.>")
                        }
                    },
                    None => String::from("No path selected"),
                };

                Column::new()
                    .width(Length::Shrink)
                    .push(Text::new("First, you will have to select your league directory"))
                    .push( button(&mut btn_states.choose_dir_btn, "Choose").on_press(Message::PickFolder) )
                    .push(Text::new(format!("Currently selected directory: {}", displayed_path)))
                    .push(Text::new(format!("Lockfile: {:?}", self.league_status.lockfile)))
                    .push( button(&mut btn_states.go_to_runes_btn, "View runes").on_press(Message::GoToRunes))
            }
            Screen::RunepageDisplay{overviews} => {
                let mut content_column =
                    Column::new()
                    .width(Length::Shrink)
                    .push(Text::new("Runepages:"));

                if overviews.len() > 0 {
                    content_column = overviews
                        .iter_mut()
                        .enumerate()
                        .fold(content_column, |column, (ind, overview)| {
                            column.push(overview.view().map(move |message| { Message::RunepageOverviewMessage(ind, message)}))
                        });
                } else {
                    content_column = content_column.push(Text::new("Nothing here..."));
                }
                content_column
            }
            Screen::RunepageModify {id} => {
                let rune_ids : String = if let (Some(pages), Some(runes_by_id)) = (&self.runepages, &self.runes_by_id) {
                    let runepage = pages[id.clone()].clone();
                    runepage.selectedPerkIds.iter().fold(String::new(), |mut acc, perkid| {
                        let name = match runes_by_id.get(perkid) {
                            Some(rune) => rune.name.clone(),
                            None => String::from("Rune not found"),
                        };
                        acc.push_str(&format!("{} {:?}", perkid, name));
                        acc
                    })
                } else {
                    String::from("Cant get rune_ids")
                };
                
                Column::new()
                    .width(Length::Shrink)
                    .push(Text::new(format!("Modifying runepage {}", id)))
                    .push(Text::new(rune_ids))
            }
            Screen::Normal => {
                Column::new()
                    .width(Length::Shrink)
                    .push(Text::new(format!("Normal {}", "test")))
            }
        };
        
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

fn button<'a>(state: &'a mut button::State, text: &str) -> Button<'a, Message> {
    Button::new(state, Text::new(text))
        .padding(10)
}

impl Runomicon {
    async fn pick_folder(default_path: Option<&Path>) -> Result<PathBuf, Error> {
        let response = nfd2::open_pick_folder(default_path)?;
        match response {
            nfd2::Response::Okay(path) => {
                Ok(path)
            },
                nfd2::Response::OkayMultiple(_multiple_paths) => {
                Err(Error::Simple(StringError{desc: "nfd2 returned multiple paths"}))
            },
                nfd2::Response::Cancel => {
                Err(Error::Simple(StringError{desc: "nfd2 dir selection was cancelled"}))
            },
        }
    }
    async fn get_runepages(lockfile: Lockfile) -> Result<String, Error> {

        // create the authorization header contents
        let authorization : String = {
            let user_and_password = ["riot", &lockfile.password].join(":");
            let as_b64 = base64::encode(user_and_password);
            ["Basic", &as_b64].join(" ")
        };

        // send the request to the league api
        let resp = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
            .get(&(["https://127.0.0.1:", &lockfile.port.to_string(), "/lol-perks/v1/pages"].join("")))
            .header("Authorization",  authorization)
            .send()
            .await?;

        let text =
            resp
            .text()
            .await?;

        Ok(text)
    }
    async fn get_runes(lockfile: Lockfile) -> Result<String, Error> {

        // create the authorization header contents
        let authorization : String = {
            let user_and_password = ["riot", &lockfile.password].join(":");
            let as_b64 = base64::encode(user_and_password);
            ["Basic", &as_b64].join(" ")
        };

        // send the request to the league api
        let resp = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
            .get(&(["https://127.0.0.1:", &lockfile.port.to_string(), "/lol-perks/v1/perks"].join("")))
            .header("Authorization",  authorization)
            .send()
            .await?;

        let text =
            resp
            .text()
            .await?;

        Ok(text)
    }
}

#[derive(Debug, Clone)]
struct RunepageOverview {
    name: String,
    state : RunepageOverviewState
}

#[derive(Debug, Clone)]
enum RunepageOverviewState {
    Idle {
        modify_button: button::State,
    }
}

#[derive(Debug, Clone)]
enum RunepageOverviewMessage {
    Modify,
}

impl RunepageOverview {
    fn new(name: String) -> Self {
        RunepageOverview {
            name,
            state: RunepageOverviewState::Idle {
                modify_button: button::State::new(),
            },
        }
    }

    fn update(&mut self, message: RunepageOverviewMessage) {
        match message {
            RunepageOverviewMessage::Modify => {},
        }
    }

    fn view(&mut self) -> Element<RunepageOverviewMessage> {
        match &mut self.state {
            RunepageOverviewState::Idle {modify_button} => {
                Row::new()
                    .push(Text::new(self.name.clone()))
                    .push(Button::new(modify_button, Text::new("Modify")).on_press(RunepageOverviewMessage::Modify))
                    .into()
                    
            }
        }
    }
}

impl From<&LolPerksPerkPage> for RunepageOverview {
    fn from(perkpage : &LolPerksPerkPage) -> Self {
        Self {
            name: perkpage.name.clone(),
            state: RunepageOverviewState::Idle {modify_button: button::State::new()}
        }
    }
}
