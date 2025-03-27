use std::error::Error;
use yew::prelude::*; // Includes function_component, html, Callback, etc.
use yew::events::InputEvent;
use web_sys::HtmlInputElement;




#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Debug, Clone)] // Clone needed for Yew state
struct Character {
    Name: String,
    Gun: String,
    ReloadTime: f32,
    Ammo: u32,
    RoF: u32,
    BurstValue: f32,
    frames: Vec<f32>,
}

fn load_characters() -> Result<Vec<Character>, Box<dyn Error>> {
    let csv_data = include_str!("../characters.csv");
    let mut rdr = csv::Reader::from_reader(csv_data.as_bytes());
    let headers = rdr.headers()?.clone();
    let mut characters = Vec::new();

    for (row, result) in rdr.records().enumerate() {
        let record = result?;
        let mut frames: Vec<f32> = Vec::new();
        for (j, header) in headers.iter().enumerate() {
            if header.starts_with("F") {
                let value = record.get(j).unwrap_or("0").parse::<f32>().unwrap_or_else(|e| {
                    eprintln!("Row {}: Failed to parse frame '{}' at column {}: {}", row, record.get(j).unwrap_or(""), header, e);
                    0.0
                });
                frames.push(value);
            }
        }
        let character = Character {
            Name: record.get(0).unwrap_or("").to_string(),
            Gun: record.get(1).unwrap_or("").to_string(),
            ReloadTime: record.get(7).unwrap_or("0").parse::<f32>().unwrap_or_else(|e| {
                eprintln!("Row {}: Failed to parse ReloadTime '{}': {}", row, record.get(7).unwrap_or(""), e);
                0.0
            }),
            Ammo: record.get(8).unwrap_or("0").parse::<u32>().unwrap_or(0),
            RoF: record.get(9).unwrap_or("0").parse::<u32>().unwrap_or(0),
            BurstValue: record.get(11).unwrap_or("0").parse::<f32>().unwrap_or_else(|e| {
                eprintln!("Row {}: Failed to parse BurstValue '{}': {}", row, record.get(11).unwrap_or(""), e);
                0.0
            }),
            frames,
        };
        characters.push(character);
    }
    Ok(characters) // Return the parsed characters
}

fn simulate_burst_gauge(team: &[&Character]) -> Result<(usize, f32), String> {
    for frame in 0..150 {
        let frame_sum: f32 = team.iter().map(|c| c.frames[frame]).sum();
        if frame_sum >= 100.0 {
            let time = frame as f32 / 30.0;
            return Ok((frame + 1, time));
        }
    }
    Err("Team probably won't burst".to_string())
}

#[allow(dead_code)]
fn adjust_burst_generation(character: &Character, charge_bonus: f32) -> Vec<f32> {
    if !character.Gun.contains("RL") && !character.Gun.contains("SR") {
        return character.frames.clone();
    }
    let new_reload = character.ReloadTime * (1.0 - charge_bonus / 100.0);
    let shots = (5.0 / new_reload).ceil() as usize;
    let burst_per_shot = character.BurstValue;
    let mut new_frames = vec![0.0; 150];
    for i in 0..shots {
        let fire_time = i as f32 * new_reload;
        if fire_time >= 5.0 {
            break;
        }
        let frame = (fire_time * 30.0).floor() as usize;
        if frame < 150 {
            new_frames[frame] = burst_per_shot;
        }
    }
    new_frames
}

#[function_component(App)]
fn app() -> Html {
    let characters = use_state(|| load_characters().unwrap_or_else(|_| Vec::new()));
    let search_query = use_state(|| "".to_string());
    let team_a = use_state(|| Vec::<Character>::new());
    let team_b = use_state(|| Vec::<Character>::new());
    let team_a_result = use_state(|| None::<Result<(usize, f32), String>>);
    let team_b_result = use_state(|| None::<Result<(usize, f32), String>>);

    let on_search = {
        let search_query = search_query.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            search_query.set(input.value());
        })
    };

    let on_search_keypress = {
        let search_query = search_query.clone();
        let team_a = team_a.clone();
        let characters = characters.clone();
        let team_b = team_b.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                let filtered = (*characters)
                    .iter()
                    .filter(|c| c.Name.to_lowercase().contains(&(*search_query).to_lowercase()))
                    .collect::<Vec<_>>();
                if let Some(first_match) = filtered.first() {
                    let character = (*first_match).clone();
                    // Try Team A first
                    let mut current_team_a = (*team_a).clone();
                    if current_team_a.len() < 5 && !current_team_a.iter().any(|c| c.Name == character.Name) {
                        current_team_a.push(character.clone());
                        team_a.set(current_team_a);
                        search_query.set("".to_string()); // Clear search input
                    } else {
                        // If Team A is full, try Team B
                        let mut current_team_b = (*team_b).clone();
                        if current_team_b.len() < 5 && !current_team_b.iter().any(|c| c.Name == character.Name) {
                            current_team_b.push(character.clone());
                            team_b.set(current_team_b);
                            search_query.set("".to_string()); // Clear search input
                        }
                    }
                }
            }
        })
    };

    let on_select_team_a = {
        let team_a = team_a.clone();
        Callback::from(move |character: Character| {
            let mut current_team = (*team_a).clone();
            if current_team.len() < 5 && !current_team.iter().any(|c| c.Name == character.Name) {
                current_team.push(character.clone());
                team_a.set(current_team);
            }
        })
    };

    let on_select_team_b = {
        let team_b = team_b.clone();
        Callback::from(move |character: Character| {
            let mut current_team = (*team_b).clone();
            if current_team.len() < 5 && !current_team.iter().any(|c| c.Name == character.Name) {
                current_team.push(character.clone());
                team_b.set(current_team);
            }
        })
    };

    let on_simulate = {
        let team_a = team_a.clone();
        let team_b = team_b.clone();
        let team_a_result = team_a_result.clone();
        let team_b_result = team_b_result.clone();
        Callback::from(move |_| {
            if (*team_a).len() == 5 && (*team_b).len() == 5 {
                let team_a_refs: Vec<&Character> = (*team_a).iter().collect();
                let team_b_refs: Vec<&Character> = (*team_b).iter().collect();
                team_a_result.set(Some(simulate_burst_gauge(&team_a_refs)));
                team_b_result.set(Some(simulate_burst_gauge(&team_b_refs)));
            }
        })
    };

    let on_remove_team_a = {
        let team_a = team_a.clone();
        Callback::from(move |index: usize| {
            let mut current_team = (*team_a).clone();
            if index < current_team.len() {
                current_team.remove(index);
                team_a.set(current_team);
            }
        })
    };
    
    let on_remove_team_b = {
        let team_b = team_b.clone();
        Callback::from(move |index: usize| {
            let mut current_team = (*team_b).clone();
            if index < current_team.len() {
                current_team.remove(index);
                team_b.set(current_team);
            }
        })
    };

    let filtered_characters = (*characters)
        .iter()
        .filter(|c| c.Name.to_lowercase().contains(&(*search_query).to_lowercase()))
        .collect::<Vec<_>>();

    html! {
        <div class="container">
            <div class="character-list">
            <input type="text" placeholder="Search characters" oninput={on_search} onkeypress={on_search_keypress} />
            <ul>
                { for filtered_characters.iter().map(|c| {
                    let character = (*c).clone();           // Base clone from the iterator
                    let character_for_a = character.clone(); // Clone for Team A
                    let character_for_b = character.clone(); // Clone for Team B
                    html! {
                    <li>
                    { &character.Name }
                    <li>
                    <button class="team-button" onclick={on_select_team_a.reform(move |_| character_for_a.clone())}>
                        {"Team A"}
                    </button>
                    <button class="team-button" onclick={on_select_team_b.reform(move |_| character_for_b.clone())}>
                        {"Team B"}
                    </button>
                    </li>
                </li>
                }
            })}
            </ul>
            </div>
            <div class="team-section">
    <h3>{"Team A"}</h3>
    <ul>
        { for team_a.iter().enumerate().map(|(i, c)| {
            html! {
                <li>
                    { &c.Name }
                    <button onclick={on_remove_team_a.reform(move |_| i)}>{"Remove"}</button>
                </li>
            }
        })}
    </ul>
</div>
<div class="team-section">
    <h3>{"Team B"}</h3>
    <ul>
        { for team_b.iter().enumerate().map(|(i, c)| {
            html! {
                <li>
                    { &c.Name }
                    <button onclick={on_remove_team_b.reform(move |_| i)}>{"Remove"}</button>
                </li>
            }
        })}
    </ul>
</div>
            
            <button onclick={on_simulate}>{"Simulate"}</button>
            <div>
                <h3>{"Team A Result"}</h3>
                { match &*team_a_result {
                    Some(Ok((frame, time))) => html! { <p>{ format!("Burst at frame {} ({}s)", frame, time) }</p> },
                    Some(Err(msg)) => html! { <p>{ msg }</p> },
                    None => html! { <p>{"Not simulated"}</p> },
                }}
            </div>
            <div>
                <h3>{"Team B Result"}</h3>
                { match &*team_b_result {
                    Some(Ok((frame, time))) => html! { <p>{ format!("Burst at frame {} ({}s)", frame, time) }</p> },
                    Some(Err(msg)) => html! { <p>{ msg }</p> },
                    None => html! { <p>{"Not simulated"}</p> },
                }}
            </div>
            <div>
                <h3>{"Comparison"}</h3>
                { match (&*team_a_result, &*team_b_result) {
                    (Some(Ok((_, time_a))), Some(Ok((_, time_b)))) => {
                        if time_a < time_b {
                            html! { <p>{"Team A bursts first"}</p> }
                        } else if time_b < time_a {
                            html! { <p>{"Team B bursts first"}</p> }
                        } else {
                            html! { <p>{"Both teams burst at the same time"}</p> }
                        }
                    },
                    (Some(Ok(_)), Some(Err(_))) => html! { <p>{"Team A bursts, Team B does not"}</p> },
                    (Some(Err(_)), Some(Ok(_))) => html! { <p>{"Team B bursts, Team A does not"}</p> },
                    (Some(Err(_)), Some(Err(_))) => html! { <p>{"Neither team bursts"}</p> },
                    _ => html! { <p>{"Simulate both teams"}</p> },
                }}
            </div>
            <div class="timeline">
                { if let Some(Ok((_, time_a))) = &*team_a_result {
                    html! {
                        <div class="team-bar team-a" style={format!("width: {}px;", (time_a * 60.0).min(300.0))}></div>
                    }
                } else {
                    html! {}
                }}
                { if let Some(Ok((_, time_b))) = &*team_b_result {
                    html! {
                        <div class="team-bar team-b" style={format!("width: {}px; margin-top: 10px;", (time_b * 60.0).min(300.0))}></div>
                    }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}