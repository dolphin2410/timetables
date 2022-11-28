use comcigan_rs::{client::WasmClient, init, search_school, view, class::SchoolData, School};
use web_sys::{HtmlInputElement, HtmlElement, window};
use yew::prelude::*;

enum Msg {
    SetValue { data: SchoolData },
    AddClass { new_class: String },
    SearchSchool { text: String },
    AddSchools { schools: Vec<School> },
    SetInput { input: HtmlInputElement },
    ViewStage { custom: Option<String> },
    UpdateStage { stage: Stage },
    UpdateSchoolClass { grade: u8, class: u8 },
}

#[derive(PartialEq, Clone)]
enum Stage {
    View,
    Search,
    Load
}

struct Model {
    value: SchoolData,
    stage: Stage,
    class: Classes,
    school: Vec<School>,
    input: Option<HtmlInputElement>,
    grade: u8,
    school_class: u8
}

fn set_local_storage<'a>(key: &'a str, value: &'a str) {
    window().unwrap().local_storage().unwrap().unwrap().set_item(key, value).unwrap();
}

fn get_local_storage(key: &str) -> Option<String> {
    window().unwrap().local_storage().unwrap().unwrap().get_item(key).unwrap()
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut instance = Self {
            value: SchoolData::new("INVALID"),
            stage: Stage::Load,
            class: Classes::default(),
            school: vec![],
            input: None,
            grade: 1,
            school_class: 1
        };

        if let Some(value) = get_local_storage("school_name") {
            ctx.link().callback(move |_| Msg::ViewStage { custom: Some(value.clone()) }).emit(());
        } else {
            instance.stage = Stage::Search;
        }

        if let Some(value) = get_local_storage("grade") {
            instance.grade = value.parse().unwrap();
        }

        if let Some(value) = get_local_storage("class") {
            instance.school_class = value.parse().unwrap();
            log::info!("Class: {}", value);
        }

        instance
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let link = ctx.link().to_owned();
        match msg {
            Msg::SetValue { data } => {
                if let Some(old_school_name) = get_local_storage("school_name") {
                    if old_school_name != data.name {
                        self.school_class = 1;
                        self.grade = 1;
                    }
                } else {
                    self.school_class = 1;
                    self.grade = 1;
                }

                set_local_storage("school_name", &data.clone().name);
                self.value = data;
                self.school.clear();
                self.input = None;
                self.stage = Stage::View;
                true
            },
            Msg::AddClass { new_class } => {
                self.class.extend([new_class]);
                true
            },
            Msg::SearchSchool { text } => {
                wasm_bindgen_futures::spawn_local(async move {
                    let client = WasmClient::new(String::from("http://corsproxy.dolphin2410.me/"));
                    let keys = init(&client).await.unwrap();
                    let schools = search_school(&client, text.as_str(), &keys).await.unwrap();
                    link.callback(move |_| Msg::AddSchools { schools: schools.clone() }).emit(());
                });
                true
            },
            Msg::AddSchools { schools } => {
                self.school = schools;
                true
            },
            Msg::SetInput { input } => {
                self.input = Some(input);
                true
            },
            Msg::ViewStage { custom } => {
                let input = self.input.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let client = WasmClient::new(String::from("http://corsproxy.dolphin2410.me/"));
                    let keys = init(&client).await.unwrap();
                    let schools = if let Some(value) = custom {
                        search_school(&client, value.as_str(), &keys).await.unwrap()
                    } else {
                        search_school(&client, input.unwrap().value().as_str(), &keys).await.unwrap()
                    };
                    let school = view(&client, &schools[0], &keys).await.unwrap();
                    link.callback(move |_| Msg::SetValue { data: school.clone() }).emit(());
                });
                true
            },
            Msg::UpdateStage { stage } => {
                self.stage = stage;
                true
            },
            Msg::UpdateSchoolClass { grade, class } => {
                if grade != 0 {
                    self.grade = grade;
                    set_local_storage("grade", grade.to_string().as_str());
                }
                if class != 0 {
                    self.school_class = class;
                    set_local_storage("class", class.to_string().as_str());
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link().to_owned();
        let click_callback = Callback::from(move |_| {
            link.callback(|_| Msg::AddClass { new_class: String::from("input-toggle") }).emit(());
        });
        let link = ctx.link().to_owned();
        let input_callback = Callback::from(move |e: KeyboardEvent| {
            if e.key().as_str() == "Enter" {
                let input_html = e.target_dyn_into::<HtmlInputElement>().unwrap();
                let text = input_html.value();
                
                link.callback(move |_| Msg::SearchSchool { text: text.clone() }).emit(());
    
                link.callback(move|_| Msg::SetInput { input: input_html.clone() }).emit(());
                // link.callback(|_| Msg::ViewStage { custom: None }).emit(());
                // return
            }
        });

        let link = ctx.link().to_owned();
        let onclick_school = Callback::from(move |e: MouseEvent| {
            let element = e.target_dyn_into::<HtmlElement>().unwrap();
            if element.inner_html() != "없으면 추가 검색하세요" {
                link.callback(move |_| Msg::ViewStage { custom: Some(element.inner_html()) }).emit(());
            }
        });
        let link = ctx.link().to_owned();
        let change_school = Callback::from(move |_| {
            link.callback(|_| Msg::UpdateStage { stage: Stage::Search }).emit(());
        });

        let link = ctx.link().to_owned();
        let edit_grade = Callback::from(move |e: KeyboardEvent| {
            let el = e.target_dyn_into::<HtmlElement>().unwrap().inner_html();
            if e.key() == "Enter" {
                e.prevent_default();
                link.callback(move |_| Msg::UpdateSchoolClass { grade: el.parse().unwrap(), class: 0 }).emit(());
            }
        });

        let link = ctx.link().to_owned();
        let edit_class = Callback::from(move |e: KeyboardEvent| {
            let el = e.target_dyn_into::<HtmlElement>().unwrap().inner_html();
            if e.key() == "Enter" {
                e.prevent_default();
                log::info!("Changed Class");
                link.callback(move |_| Msg::UpdateSchoolClass { grade: 0, class: el.parse().unwrap() }).emit(());
            }
        });
        match self.stage {
            Stage::Load => html! {
                <div class="loading">
                    <h1>{"Loading..."}</h1>
                </div>
            },
            Stage::Search => html! {
                <div class="container">
                    <div class="incontainer flex">
                        <div class="container-horizontal-left"><h3>{"학교검색"}</h3></div>
                        <div class={classes!("search-area", self.class.clone())}>
                            <div class="input_group">
                            <input id="search_input" type="text" onclick={click_callback} onkeyup={input_callback} />
                            // <button id="search_button" type="button">
                                //     <i class="fa-solid fa-magnifying-glass"></i>
                                // </button>
                            </div>
                            <div class="school_list container-horizontal-left">
                                {
                                    self.school.iter().map(|school| {
                                        html! {
                                            <div onclick={onclick_school.clone()}>
                                                {school.2.clone()}
                                            </div>
                                        }
                                    }).collect::<Html>()
                                }
                            </div>
                        </div>
                    </div>
                </div>
            },
            Stage::View => html! {
                <div class="container">
                <table class="incontainer">
                <tr>
                <td align="center" colspan="5"><h3>{self.value.name.clone()} { " " } <span class="no-outline" contenteditable="true" onkeydown={edit_grade}>{ self.grade }</span> {"학년 "} <span class="no-outline" contenteditable="true" onkeypress={edit_class}>{ self.school_class }</span>{"반"}</h3></td>
            </tr>
                    <tr>
                        <th>{ "월" }</th>
                        <th>{ "화" }</th>
                        <th>{ "수" }</th>
                        <th>{ "목" }</th>
                        <th>{ "금" }</th>
                    </tr>
                        { 
                            (1..=7).into_iter().map(|period| {
                                html! {
                                <tr>
                                    {
                                        (1..=5).into_iter().map(|day| {html! {
                                            <td>{ self.value.clone().grade(self.grade as usize).class(self.school_class as usize).day(day).period(period).subject }</td>
                                        }}).collect::<Html>()
                                    }
                                </tr>
                            }
                        }).collect::<Html>()
                        }
                    <tr>
                        <td align="center" colspan="5"><button id="change-school" onclick={change_school}>{ "학교 변경하기" }</button></td>
                    </tr>
                </table>
            </div>
            }
        }
        
    }
}

fn main() {
    // wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Model>();
}