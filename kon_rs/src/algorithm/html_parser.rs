use scraper::{selector::CssLocalName, CaseSensitivity, Element, Html, Selector};

use super::BandSchedule;

pub struct HtmlParser;

impl HtmlParser {
    pub fn parse_band_schedule(html: &str) -> BandSchedule {
        let fragment = Html::parse_fragment(html);
        let selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        let mut name = String::new();
        let mut is_available_data = Vec::default();
        let mut members = Vec::default();

        for element in fragment.select(&selector) {
            let mut td_iterator = element.select(&td_selector).peekable();

            // 最初はバンマス
            name = td_iterator
                .next()
                .unwrap()
                .children()
                .next()
                .unwrap()
                .children()
                .next()
                .unwrap()
                .value()
                .as_text()
                .unwrap()
                .to_string();

            // 予定の抽出
            loop {
                let Some(td) = td_iterator.peek() else {
                    break;
                };
                let ul_selector = Selector::parse("ul").unwrap();
                let Some(ul) = td.select(&ul_selector).next() else {
                    break;
                };

                let li_selector = Selector::parse("li").unwrap();
                let li = ul.select(&li_selector).next().unwrap();
                let is_available = li.has_class(
                    &CssLocalName::from("checked"),
                    CaseSensitivity::AsciiCaseInsensitive,
                );
                is_available_data.push(is_available);

                // 消費
                td_iterator.next().unwrap();
            }

            // メンバーの抽出
            let Some(td) = td_iterator.next() else {
                break;
            };
            let a_selector = Selector::parse("a").unwrap();
            let a_elements = td.select(&a_selector);
            for a_element in a_elements {
                let member = a_element.value().attr("data-username").unwrap();
                members.push(member.to_string());
            }
        }

        BandSchedule {
            name,
            is_available: is_available_data,
            members,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HtmlParser;

    #[test]
    fn hello() {
        let band_schedule = HtmlParser::parse_band_schedule(
            r#"
            <table>
                <tr>
                    <td class="title" data-content-id="2732315951"><a href="/confluence/pages/viewpage.action?pageId=2732315951">江戸川コナン</a></td>
                    <td>
                        <ul class="inline-task-list" data-inline-tasks-content-id="2732315951">
                            <li class="checked" data-inline-task-id="72"><span>&nbsp;</span></li>
                        </ul>
                    </td>
                    <td>
                        <ul class="inline-task-list" data-inline-tasks-content-id="2807657106">
                            <li data-inline-task-id="73"><span>&nbsp;</span></li>
                        </ul>
                    </td>
                    <td>
                        <div class="content-wrapper">
                            <p>
                                <a data-username="edogawa_conan">江戸川　コナン</a>
                                <br />
                                <a data-username="akai_shuichi">赤井　秀一</a>
                                <br />
                                <a data-username="mori_kogoro" href="/confluence/display/~mori_kogoro">毛利　小五郎</a>
                                &nbsp;</p>
                        </div>
                    </td>
                    </tr>
            </table>
            "#,
        );

        // バンマス名
        assert_eq!(band_schedule.name, "江戸川コナン");

        // 予定
        assert_eq!(band_schedule.is_available.len(), 2);
        assert!(band_schedule.is_available[0]);
        assert!(!band_schedule.is_available[1]);

        // メンバー
        assert_eq!(band_schedule.members.len(), 3);
        assert_eq!(band_schedule.members[0], "edogawa_conan");
        assert_eq!(band_schedule.members[1], "akai_shuichi");
        assert_eq!(band_schedule.members[2], "mori_kogoro");
    }
}
