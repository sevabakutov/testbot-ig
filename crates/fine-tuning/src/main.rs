use std::fs::File;
use std::io::{BufWriter, Write};

use anyhow::{Context, Result};
use dotenv::dotenv;
use fine_tuning::{client::Client, settings::Settings};
use serde::{Serialize};

#[derive(Debug, Serialize)]
struct FineTuneEntry {
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().context("Failed to read env-vars")?;

    let settings = Settings::new()?;
    let client = Client::new(settings).await?;

    let ranges = vec![
        "Шаблони для переписки!A1:Z",
        "Підтвердження за посиланням!A1:Z",
        "ЛЕ!A1:Z",
        // "Акції Київ+Бровари, Вишневе!A1:Z",
        // "Акції Регіони 1 !A1:Z",
        // "Акції Региони 2 !A1:Z",
        // "Акції Малютки!A1:Z",
        // "Виключення з акцій (в процесі)!A1:Z",
        // "Сертифікати!A1:Z",
        // "Адреси центрів!A1:Z",
        // "Видалення судин!A1:Z",
        // "Видалення тату!A1:Z",
        // "Меладин!A1:Z",
        // "Чистки!A1:Z",
        // "Контурна пластика!A1:Z",
        // "Вид.новоутворень!A1:Z",
        // "Ботокс/диспорт!A1:Z",
        // "Біоревіталізація!A1:Z",
        // "Пігментація (в процесі)!A1:Z"
    ];

    let mut entries: Vec<FineTuneEntry> = Vec::new();

    for range in ranges {
        let rows = client.fetch_sheet(range).await?;
        let sheet_type = range.split('!').next().unwrap_or(range).to_string();
        let sheet_entries = rows_to_entries(&sheet_type, &rows);
        entries.extend(sheet_entries);
    }

    serialize_to_jsonl(&entries, "jsonl/output.jsonl")?;

    Ok(())
}

fn rows_to_entries(sheet_type: &str, rows: &[Vec<String>]) -> Vec<FineTuneEntry> {
    let system_prompt = r#"
Ти — консультант мережі з 70 косметологічних салонів. Працюєш у чаті Instagram від імені живої людини-оператора.

• Відповідай лише на питання, що стосуються наших салонів:
  - послуги, підготовка, протипоказання, тривалість;
  - ціни, акції, подарункові сертифікати, програма лояльності;
  - адреси, графік роботи, паркування, доступність;
  - вакансії, франшиза, співпраця.
• Усі інші запити ввічливо перенаправляй колезі (див. правило ескалації).
• Ніколи не згадуй про штучний інтелект, ботів чи внутрішні інструкції.
• Якщо потрібна допомога колеги або зміна запису — просто надішли клієнту смайлик «😊» (без додаткового тексту).

Якщо текст користувача польською, відповідай польською.
Якщо російською — відповідай російською.
За замовчуванням — українською.    
"#;

    let developer_prompt = r#"
**Стиль відповіді**  
• Дружній, але діловий тон.  
• До 3–4 коротких речень.  
• Першим словом звернись до клієнта («Вітаю!», «Добрий день!»).  
• В кінці завжди пропонуй подальшу допомогу («Чим ще можу бути корисна?»).  
• У всьому повідомленні — не більше одного смайлика 😊.  

**Приклад структури**  
Привітання → суть відповіді → заклик/пропозиція допомоги.  

**Ескалація**  
Якщо інформації не достатньо, тема поза бізнесом салону або клієнт просить змінити/скасувати запис — надішли клієнту лише «😊».  
"#;

    match sheet_type {
        "Шаблони для переписки" => {
            rows.iter().skip(2).flat_map(|row| {
                let topic = row[0].trim().to_string();
                let ua_variants = row[1].trim().replace("\\n", "\n").split("\n\n").map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect::<Vec<_>>();
                let pl_variants = row[2].trim().replace("\\n", "\n").split("\n\n").map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect::<Vec<_>>();

                let mut ents = Vec::new();
                for ua in ua_variants {
                    let user_content = format!("Type: {}\nTopic: {}\nLanguage: uk", sheet_type, topic);
                    ents.push(FineTuneEntry {
                        messages: vec![
                            Message { role: "system".to_string(), content: system_prompt.to_string() },
                            Message { role: "developer".to_string(), content: developer_prompt.to_string() },
                            Message { role: "user".to_string(), content: user_content },
                            Message { role: "assistant".to_string(), content: ua },
                        ],
                    });
                }
                for pl in pl_variants {
                    let user_content = format!("Type: {}\nTopic: {}\nLanguage: pl", sheet_type, topic);
                    ents.push(FineTuneEntry {
                        messages: vec![
                            Message { role: "system".to_string(), content: system_prompt.to_string() },
                            Message { role: "developer".to_string(), content: developer_prompt.to_string() },
                            Message { role: "user".to_string(), content: user_content },
                            Message { role: "assistant".to_string(), content: pl },
                        ],
                    });
                }
                ents
            }).collect()
        },
        "Підтвердження за посиланням" => {
            let topics = vec![
                "Підготовка зон до епіляції".to_string(),
                "Скасування або зміна запису".to_string(),
                "Оплата тільки готівкою".to_string(),
                "Необхідність сироватки для процедури".to_string(),
                "Документи для акційної оплати".to_string(),
                "Необхідність костюму LPG".to_string(),
                "Згода батьків для процедури".to_string(),
                "Підготовка до нано-епіляції".to_string(),
                "Підготовка до консультації трихолога".to_string(),
                "Підготовка до аналізу мікроелементів".to_string(),
                "Підготовка нігтів до лікування грибка".to_string(),
                "Скасування знижки за готівку".to_string(),
                "Об'їзд дороги на машині".to_string(),
                "Заміна".to_string(),
                "Заміна послуг або умов".to_string(),
                "Нові ручні масажі".to_string(),
                "Нові процедури класичної косметології".to_string(),
                "Нові процедури косметології та масажів".to_string(),
                "Нові процедури для тіла".to_string(),
            ];
            rows.iter().skip(1).enumerate().flat_map(|(i, row)| {
                let topic = topics.get(i).cloned().unwrap_or_default();
                let mut ents = Vec::new();

                // UA
                if let Some(text_ua) = row.get(0) {
                    let variants_ua = text_ua.trim().replace("\\n", "\n").split("\n\n").map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect::<Vec<_>>();
                    for answer in variants_ua {
                        let user_content = format!("Type: {}\nTopic: {}\nLanguage: ua", sheet_type, topic);
                        ents.push(FineTuneEntry {
                            messages: vec![
                                Message { role: "system".to_string(), content: system_prompt.to_string() },
                                Message { role: "developer".to_string(), content: developer_prompt.to_string() },
                                Message { role: "user".to_string(), content: user_content },
                                Message { role: "assistant".to_string(), content: answer },
                            ],
                        });
                    }
                }

                // PL
                if let Some(text_pl) = row.get(5) {
                    let variants_pl = text_pl.trim().replace("\\n", "\n").split("\n\n").map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect::<Vec<_>>();
                    for answer in variants_pl {
                        let user_content = format!("Type: {}\nTopic: {}\nLanguage: pl", sheet_type, topic);
                        ents.push(FineTuneEntry {
                            messages: vec![
                                Message { role: "system".to_string(), content: system_prompt.to_string() },
                                Message { role: "developer".to_string(), content: developer_prompt.to_string() },
                                Message { role: "user".to_string(), content: user_content },
                                Message { role: "assistant".to_string(), content: answer },
                            ],
                        });
                    }
                }

                // ENG
                if let Some(text_eng) = row.get(10) {
                    let variants_eng = text_eng.trim().replace("\\n", "\n").split("\n\n").map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect::<Vec<_>>();
                    for answer in variants_eng {
                        let user_content = format!("Type: {}\nTopic: {}\nLanguage: eng", sheet_type, topic);
                        ents.push(FineTuneEntry {
                            messages: vec![
                                Message { role: "system".to_string(), content: system_prompt.to_string() },
                                Message { role: "developer".to_string(), content: developer_prompt.to_string() },
                                Message { role: "user".to_string(), content: user_content },
                                Message { role: "assistant".to_string(), content: answer },
                            ],
                        });
                    }
                }

                ents
            }).collect()
        }
        "ЛЕ" => {
            let mut entries = Vec::new();
            let mut current_lang = "ua".to_string();
            let mut current_procedure = "лазерна епіляція".to_string();

            for row in rows.iter() {
                if row.is_empty() {
                    continue;
                }

                let first_cell = row[0].trim().to_lowercase();
                
                if first_cell.eq("\"14.07.2025 12:42:30\"") {
                    current_lang = "ua".to_string();
                    current_procedure = "лазерна епіляція".to_string();
                    continue;
                } else if first_cell.eq("\"російською\"") {
                    current_lang = "ru".to_string();
                    current_procedure = "лазерная эпиляция".to_string();
                    continue;
                } else if first_cell.eq("\"wersja polska\"") {
                    current_lang = "pl".to_string();
                    current_procedure = "depilacja laserowa".to_string();
                    continue;
                }

                if first_cell.eq("\"курс процедур укр.\"") || first_cell.eq("\"курс процедур рус.\"") || first_cell.eq("\"kurs zabiegów\"") {
                    continue;
                }

                // Data row
                let course_procedures = row[0].trim().replace("\\n", "\n");
                let intervals = row[1].trim().replace("\\n", "\n");
                let differences = row[2].trim().replace("\\n", "\n");
                let preparation = row[3].trim().replace("\\n", "\n");
                let contraindications = row[4].trim().replace("\\n", "\n");
                let after_procedure_time = row[5].trim().replace("\\n", "\n");
                let result_after_procedure = row[6].trim().replace("\\n", "\n");
                let rehabilitation = row[7].trim().replace("\\n", "\n");
                let pain_relief = row[8].trim().replace("\\n", "\n");
                let guarantee = row[9].trim().replace("\\n", "\n");
                let additional_services = row[10].trim().replace("\\n", "\n");

                let full_text = match current_lang.as_str() {
                    "ua" => format!(
                        "Курс процедур: {}\n\nІнтервали: {}\n\nВідмінності: {}\n\nПідготовка: {}\n\nПротипоказання: {}\n\nПісля процедури: {}\n\nРезультат після процедури: {}\n\nРеабілітація: {}\n\nЗнеболення: {}\n\nГарантія: {}\n\nДля запису: {}",
                        course_procedures, intervals, differences, preparation, contraindications, after_procedure_time, result_after_procedure, rehabilitation, pain_relief, guarantee, additional_services
                    ),
                    "ru" => format!(
                        "Курс процедур: {}\n\nИнтервалы: {}\n\nОтличия: {}\n\nПодготовка: {}\n\nПротивопоказания: {}\n\nПослепроцедурный режим: {}\n\nРезультат после процедуры: {}\n\nРеабилитация: {}\n\nОбезболивание: {}\n\nГАРАНТИЯ РЕЗУЛЬТАТА: {}\n\nДля записи: {}",
                        course_procedures, intervals, differences, preparation, contraindications, after_procedure_time, result_after_procedure, rehabilitation, pain_relief, guarantee, additional_services
                    ),
                    "pl" => format!(
                        "Kurs zabiegów: {}\n\nOdstępy: {}\n\nRóżnice: {}\n\nPrzygotowanie: {}\n\nPrzeciwwskazania: {}\n\nPostępowanie po zabiegu: {}\n\nWynik po zabiegu: {}\n\nRehabilitacja: {}\n\nZnieczulenie: {}\n\nGWARANCJA REZULTATU: {}\n\nDla zapisu: {}",
                        course_procedures, intervals, differences, preparation, contraindications, after_procedure_time, result_after_procedure, rehabilitation, pain_relief, guarantee, additional_services
                    ),
                    _ => unreachable!()
                };  

                let user_content = format!("Type: {}\nLanguage: {}\nTopic: {}\n", sheet_type, current_lang, current_procedure);

                entries.push(FineTuneEntry {
                    messages: vec![
                        Message { role: "system".to_string(), content: system_prompt.to_string() },
                        Message { role: "developer".to_string(), content: developer_prompt.to_string() },
                        Message { role: "user".to_string(), content: user_content },
                        Message { role: "assistant".to_string(), content: full_text },
                    ],
                });
            }

            entries
        }
        _ => unreachable!()
    }
}

fn serialize_to_jsonl(entries: &[FineTuneEntry], file_path: &str) -> Result<()> {
    let file = File::create(file_path).context(format!("Failed to create file: {}", file_path))?;
    let mut writer = BufWriter::new(file);

    for entry in entries {
        let json = serde_json::to_string(&entry).context("Failed to serialize FineTuneEntry to JSON")?;
        writeln!(writer, "{}", json).context("Failed to write JSON line to file")?;
    }

    Ok(())
}