use backend::models::burrow::BurrowShowResponse;
use backend::models::content::PostPage;
use backend::models::error::*;
use backend::models::search::*;
use backend::utils::mq::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::Status;
use serde_json::json;
use tests_integration::get_client;
use tokio::runtime::Runtime;

#[test]
fn test_search() {
    // ---------- Prepare ----------
    // Init background task executor
    let client = get_client().lock();
    let rt = Runtime::new().unwrap();
    let h2 = rt.spawn(pulsar_relation());
    let h3 = rt.spawn(pulsar_typesense());
    let h4 = rt.spawn(pulsar_email());
    std::thread::sleep(std::time::Duration::from_secs(1));
    // generate a random name
    let name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(14)
        .collect();
    // ---------- Prepare ----------

    // set verification code
    client
        .post("/users/email")
        .json(&json!({
            "email": format!("{}@mails.tsinghua.edu.cn", name)
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    std::thread::sleep(std::time::Duration::from_secs(1));
    // sign up a user
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": name,
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", name),
            "verification_code": "666666"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::models::user::UserResponse>()
        .unwrap();
    let burrow_id = res.default_burrow;

    // user login
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": name,
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // println!("{}", response.into_string().unwrap());
    std::thread::sleep(std::time::Duration::from_secs(2));

    // create burrow
    let response = client
        .post("/burrows")
        .json(&json!({
            "description": format!("Created burrow of {}", name),
            "title": "Created Burrow"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<serde_json::Value>().unwrap();
    let created_burrow_id: i64 = serde_json::to_string(&res["burrow_id"])
        .unwrap()
        .parse::<i64>()
        .unwrap();
    println!("Created Burrow Id: {}", created_burrow_id);
    // println!("{}", response.into_string().unwrap());

    // create post
    let response = client
        .post("/content/posts")
        .json(&json!({
            "title": format!("First post of {}", name),
            "burrow_id": burrow_id,
            "section": ["NSFW"],
            "tag": ["NoTag","政治相关"],
            "content": "search test"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::models::content::PostCreateResponse>()
        .unwrap();
    let post_id = res.post_id;
    println!("Post Id: {}", post_id);
    std::thread::sleep(std::time::Duration::from_secs(1));

    // retrieve burrow
    let response = client
        .post("/search")
        .json(&json!(SearchRequest::RetrieveBurrow {
            burrow_id: created_burrow_id
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<serde_json::Value>().unwrap();
    assert_eq!(res["title"], "Created Burrow".to_string());
    // println!("Retrieve result: {}", response.into_string().unwrap());

    // retrieve burrow  (invalid burrow_id)
    let response = client
        .post("/search")
        .json(&json!(SearchRequest::RetrieveBurrow { burrow_id: -1 }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    let res = response.into_json::<ErrorResponse>().unwrap();
    assert_eq!(res.error.code, ErrorCode::BurrowNotExist);
    assert_eq!(res.error.message, "Cannot find burrow -1".to_string());

    // search burrow by keyword
    let response = client
        .post("/search")
        .json(&SearchRequest::SearchBurrowKeyword {
            keywords: vec!["Created".to_string()],
        })
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<SearchBurrowResponse>().unwrap();
    assert_eq!(res.burrows[0].burrow_id, created_burrow_id);
    // println!("Search result: {}", response.into_string().unwrap());

    // // search burrow by keyword  (empty keyword vector)
    // let response = client
    //     .post(format!("/search/?{}", 1))
    //     .json(&SearchRequest::SearchBurrowKeyword { keywords: vec![] })
    //     .remote("127.0.0.1:8000".parse().unwrap())
    //     .dispatch();
    // assert_eq!(response.status(), Status::Ok);
    // println!("Search result: {}", response.into_string().unwrap());

    // search burrow by keyword  (repeat keyword vector)
    let response = client
        .post("/search")
        .json(&SearchRequest::SearchBurrowKeyword {
            keywords: vec![
                "created".to_string(),
                "created".to_string(),
                "created".to_string(),
                "created".to_string(),
                "created".to_string(),
                "created".to_string(),
                "created".to_string(),
                "created".to_string(),
                "created".to_string(),
            ],
        })
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<SearchBurrowResponse>().unwrap();
    // println!("{}",response.into_string().unwrap());
    assert_eq!(res.burrows[0].burrow_id, created_burrow_id);

    // retrieve post
    let response = client
        .post("/search")
        .json(&json!(SearchRequest::RetrievePost { post_id: 1 }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    let res = response.into_json::<PostPage>().unwrap();
    assert_eq!(res.post_desc.post_id, 1);
    // println!("Retrieve result: {}", response.into_string().unwrap());

    // search post by keyword
    let response = client
        .post("/search")
        .json(&SearchRequest::SearchPostKeyword {
            keywords: vec!["test".to_string()],
        })
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<SearchMixResponse>().unwrap();
    assert_eq!(res.replies.replies[0].post_id, post_id);

    // search post by keyword   (special characters)
    let response = client
        .post("/search")
        .json(&SearchRequest::SearchPostKeyword {
            keywords: vec!["❤❥웃유♋☮✌☏☢☠✔☑♚▲♪✈✞÷↑↓◆◇⊙■□△▽¿─│♥❣♂♀☿Ⓐ✍✉☣☤✘☒♛▼♫⌘☪≈←→◈◎☉★☆⊿※¡━┃♡ღツ☼☁❅♒✎©®™Σ✪✯☭➳卐√↖↗●◐Θ◤◥︻〖〗┄┆℃℉°✿ϟ☃☂✄¢€£∞✫★½✡×↙↘○◑⊕◣◢︼【】┅┇☽☾✚〓▂▃▄▅▆▇█▉▊▋▌▍▎▏↔↕☽☾の•▸◂▴▾┈┊①②③④⑤⑥⑦⑧⑨⑩ⅠⅡⅢⅣⅤⅥⅦⅧⅨⅩ㍿▓♨♛❖♓☪✙┉┋☹☺☻تヅツッシÜϡﭢ™℠℗©®♥❤❥❣❦❧♡۵웃유ღ♋♂♀☿☼☀☁☂☄☾☽❄☃☈⊙☉℃℉❅✺ϟ☇♤♧♡♢♠♣♥♦☜☞☝✍☚☛☟✌✽✾✿❁❃❋❀⚘☑✓✔√☐☒✗✘ㄨ✕✖✖⋆✢✣✤✥❋✦✧✩✰✪✫✬✭✮✯❂✡★✱✲✳✴✵✶✷✸✹✺✻✼❄❅❆❇❈❉❊†☨✞✝☥☦☓☩☯☧☬☸✡♁✙♆。，、＇：∶；?‘’“”〝〞ˆˇ﹕︰﹔﹖﹑•¨….¸;！´？！～—ˉ｜‖＂〃｀@﹫¡¿﹏﹋﹌︴々﹟#﹩$﹠&﹪%*﹡﹢﹦﹤‐￣¯―﹨ˆ˜﹍﹎+=<＿_-ˇ~﹉﹊（）〈〉‹›﹛﹜『』〖〗［］《》〔〕{}「」【】︵︷︿︹︽_﹁﹃︻︶︸﹀︺︾ˉ﹂﹄︼☩☨☦✞✛✜✝✙✠✚†‡◉○◌◍◎●◐◑◒◓◔◕◖◗❂☢⊗⊙◘◙◍⅟½⅓⅕⅙⅛⅔⅖⅚⅜¾⅗⅝⅞⅘≂≃≄≅≆≇≈≉≊≋≌≍≎≏≐≑≒≓≔≕≖≗≘≙≚≛≜≝≞≟≠≡≢≣≤≥≦≧≨≩⊰⊱⋛⋚∫∬∭∮∯∰∱∲∳%℅‰‱㊣㊎㊍㊌㊋㊏㊐㊊㊚㊛㊤㊥㊦㊧㊨㊒㊞㊑㊒㊓㊔㊕㊖㊗㊘㊜㊝㊟㊠㊡㊢㊩㊪㊫㊬㊭㊮㊯㊰㊙㉿囍♔♕♖♗♘♙♚♛♜♝♞♟ℂℍℕℙℚℝℤℬℰℯℱℊℋℎℐℒℓℳℴ℘ℛℭ℮ℌℑℜℨ♪♫♩♬♭♮♯°øⒶ☮✌☪✡☭✯卐✐✎✏✑✒✍✉✁✂✃✄✆✉☎☏➟➡➢➣➤➥➦➧➨➚➘➙➛➜➝➞➸♐➲➳⏎➴➵➶➷➸➹➺➻➼➽←↑→↓↔↕↖↗↘↙↚↛↜↝↞↟↠↡↢↣↤↥↦↧↨➫➬➩➪➭➮➯➱↩↪↫↬↭↮↯↰↱↲↳↴↵↶↷↸↹↺↻↼↽↾↿⇀⇁⇂⇃⇄⇅⇆⇇⇈⇉⇊⇋⇌⇍⇎⇏⇐⇑⇒⇓⇔⇕⇖⇗⇘⇙⇚⇛⇜⇝⇞⇟⇠⇡⇢⇣⇤⇥⇦⇧⇨⇩⇪➀➁➂➃➄➅➆➇➈➉➊➋➌➍➎➏➐➑➒➓㊀㊁㊂㊃㊄㊅㊆㊇㊈㊉ⒶⒷⒸⒹⒺⒻⒼⒽⒾⒿⓀⓁⓂⓃⓄⓅⓆⓇⓈⓉⓊⓋⓌⓍⓎⓏⓐⓑⓒⓓⓔⓕⓖⓗⓘⓙⓚⓛⓜⓝⓞⓟⓠⓡⓢⓣⓤⓥⓦⓧⓨⓩ⒜⒝⒞⒟⒠⒡⒢⒣⒤⒥⒦⒧⒨⒩⒪⒫⒬⒭⒮⒯⒰⒱⒲⒳⒴⒵ⅠⅡⅢⅣⅤⅥⅦⅧⅨⅩⅪⅫⅬⅭⅮⅯⅰⅱⅲⅳⅴⅵⅶⅷⅸⅹⅺⅻⅼⅽⅾⅿ┌┍┎┏┐┑┒┓└┕┖┗┘┙┚┛├┝┞┟┠┡┢┣┤┥┦┧┨┩┪┫┬┭┮┯┰┱┲┳┴┵┶┷┸┹┺┻┼┽┾┿╀╁╂╃╄╅╆╇╈╉╊╋╌╍╎╏═║╒╓╔╕╖╗╘╙╚╛╜╝╞╟╠╡╢╣╤╥╦╧╨╩╪╫╬◤◥◄►▶◀◣◢▲▼◥▸◂▴▾△▽▷◁⊿▻◅▵▿▹◃❏❐❑❒▀▁▂▃▄▅▆▇▉▊▋█▌▍▎▏▐░▒▓▔▕■□▢▣▤▥▦▧▨▩▪▫▬▭▮▯㋀㋁㋂㋃㋄㋅㋆㋇㋈㋉㋊㋋㏠㏡㏢㏣㏤㏥㏦㏧㏨㏩㏪㏫㏬㏭㏮㏯㏰㏱㏲㏳㏴㏵㏶㏷㏸㏹㏺㏻㏼㏽㏾㍙㍚㍛㍜㍝㍞㍟㍠㍡㍢㍣㍤㍥㍦㍧㍨㍩㍪㍫㍬㍭㍮㍯㍰㍘☰☲☱☴☵☶☳☷☯
            ♠♣♧♡♥❤❥❣♂♀✲☀☼☾☽◐◑☺☻☎☏✿❀№↑↓←→√×÷★℃℉°◆◇⊙■□△▽¿½☯✡㍿卍卐♂♀✚〓㎡♪♫♩♬㊚㊛囍㊒㊖Φ♀♂‖$@*&#※卍卐Ψ♫♬♭♩♪♯♮⌒¶∮‖€￡¥$
            ①②③④⑤⑥⑦⑧⑨⑩⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳⓪⓿❶❷❸❹❺❻❼❽❾❿⓫⓬⓭⓮⓯⓰⓱⓲⓳⓴⓵⓶⓷⓸⓹⓺⓻⓼⓽⓾㊀㊁㊂㊃㊄㊅㊆㊇㊈㊉㈠㈡㈢㈣㈤㈥㈦㈧㈨㈩⑴⑵⑶⑷⑸⑹⑺⑻⑼⑽⑾⑿⒀⒁⒂⒃⒄⒅⒆⒇⒈⒉⒊⒋⒌⒍⒎⒏⒐⒑⒒⒓⒔⒕⒖⒗⒘⒙⒚⒛ⅠⅡⅢⅣⅤⅥⅦⅧⅨⅩⅪⅫⅰⅱⅲⅳⅴⅵⅶⅷⅸⅹⒶⒷⒸⒹⒺⒻⒼⒽⒾⒿⓀⓁⓂⓃⓄⓅⓆⓇⓈⓉⓊⓋⓌⓍⓎⓏⓐⓑⓒⓓⓔⓕⓖⓗⓘⓙⓚⓛⓜⓝⓞⓟⓠⓡⓢⓣⓤⓥⓦⓧⓨⓩ⒜⒝⒞⒟⒠⒡⒢⒣⒤⒥⒦⒧⒨⒩⒪⒫⒬⒭⒮⒯⒰⒱⒲⒳⒴⒵
            ﹢﹣×÷±+-*/^=≌∽≦≧≒﹤﹥≈≡≠≤≥≮≯∷∶∝∞∧∨∑∏∪∩∈∵∴⊥∥∠⌒⊙√∛∜∟⊿㏒㏑%‰⅟½⅓⅕⅙⅐⅛⅑⅒⅔¾⅖⅗⅘⅚⅜⅝⅞≂≃≄≅≆≇≉≊≋≍≎≏≐≑≓≔≕≖≗≘≙≚≛≜≝≞≟≢≣≨≩⊰⊱⋛⋚∫∮∬∭∯∰∱∲∳℅øπ∀∁∂∃∄∅∆∇∉∊∋∌∍∎∐−∓∔∕∖∗∘∙∡∢∣∤∦∸∹∺∻∼∾∿≀≁≪≫≬≭≰≱≲≳≴≵≶≷≸≹≺≻≼≽≾≿⊀⊁⊂⊃⊄⊅⊆⊇⊈⊉⊊⊋⊌⊍⊎⊏⊐⊑⊒⊓⊔⊕⊖⊗⊘⊚⊛⊜⊝⊞⊟⊠⊡⊢⊣⊤⊦⊧⊨⊩⊪⊫⊬⊭⊮⊯⊲⊳⊴⊵⊶⊷⊸⊹⊺⊻⊼⊽⊾⋀⋁⋂⋃⋄⋅⋆⋇⋈⋉⋊⋋⋌⋍⋎⋏⋐⋑⋒⋓⋔⋕⋖⋗⋘⋙⋜⋝⋞⋟⋠⋡⋢⋣⋤⋥⋦⋧⋨⋩⋪⋫⋬⋭⋮⋯⋰⋱⋲⋳⋴⋵⋶⋷⋸⋹⋺⋻⋼⋽⋾⋿ⅠⅡⅢⅣⅤⅥⅦⅧⅨⅩⅪⅫⅬⅭⅮⅯↁↂↃↅↆↇↈ↉↊↋■□▢▣▤▥▦▧▨▩▪▫▬▭▮▯▰▱▲△▴▵▶▷▸▹►▻▼▽▾▿◀◁◂◃◄◅◆◇◈◉◊○◌◍◎●◐◑◒◓◔◕◖◗◘◙◚◛◜◝◞◟◠◡◢◣◤◥◦◧◨◩◪◫◬◭◮◯◰◱◲◳◴◵◶◷◸◹◺◿◻◼◽◾⏢⏥⌓⌔⌖".to_string()]
        })
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<SearchMixResponse>().unwrap();
    assert_eq!(res.posts.found, 0);
    // println!("Search result: {}", response.into_string().unwrap());

    // search post by tag
    let response = client
        .post("/search")
        .json(&json!(SearchRequest::SearchPostTag {
            tag: vec!["政治相关".to_string()]
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<SearchPostResponse>().unwrap();
    assert_eq!(res.posts[0].post_id, post_id);
    // println!("Search result: {}", response.into_string().unwrap());

    // search post by tag   (empty tag vector)
    let response = client
        .post("/search")
        .json(&SearchRequest::SearchPostTag { tag: vec![] })
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    let res = response.into_json::<ErrorResponse>().unwrap();

    assert_eq!(res.error.code, ErrorCode::EmptyField);
    assert_eq!(res.error.message, "Tags should not be empty".to_string());
    // ErrorResponse::build(ErrorCode::EmptyField,format!("Tags should not be empty")));

    // discard burrow
    let response = client
        .delete(format!("/burrows/{}", burrow_id))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_string().unwrap();
    assert_eq!(res, "Success".to_string());

    //retrieve a discarded burrow
    let response = client
        .post("/search")
        .json(&SearchRequest::RetrieveBurrow { burrow_id })
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<BurrowShowResponse>().unwrap();
    assert_eq!(res.title, "默认洞".to_string());
    // println!("Retrieve result: {}", response.into_string().unwrap());

    //retrieve a non-exist post
    let response = client
        .post("/search")
        .json(&SearchRequest::RetrievePost { post_id: -1 })
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
    let res = response.into_json::<ErrorResponse>().unwrap();
    // println!("Retrieve result: {}", response.into_string().unwrap());
    assert_eq!(res.error.code, ErrorCode::PostNotExist);
    assert_eq!(res.error.message, "Cannot find post -1".to_string());
    // user log out
    let response = client
        .get("/users/logout")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    // generate a random name
    let admin_name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(12)
        .collect();
    // Set up the admin account
    // set verification code
    client
        .post("/users/email")
        .json(&json!({
            "email": format!("{}@mails.tsinghua.edu.cn", admin_name)
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    std::thread::sleep(std::time::Duration::from_secs(1));
    // sign up a user
    let response = client
        .post("/users/sign-up")
        .json(&json!({
        "username": admin_name,
        "password": "testpassword",
        "email": format!("{}@mails.tsinghua.edu.cn", admin_name),
        "verification_code": "666666"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // user login
    let response = client
        .post("/users/login")
        .json(&json!({
        "username": admin_name,
        "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    let response = client
        .get("/admin/test?role=3")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    // Ban the burrow with burrow_id
    let response = client
        .post("/admin")
        .json(&json!({ "BanBurrow": {"burrow_id": created_burrow_id} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    let response = client
        .post("/admin")
        .json(&json!({ "BanPost": {"post_id": post_id} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    let response = client
        .post("/admin")
        .json(&json!({ "BanReply": {"post_id": post_id, "reply_id": 0} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    std::thread::sleep(std::time::Duration::from_secs(1));

    // search burrow by keyword
    let response = client
        .post("/search")
        .json(&SearchRequest::SearchBurrowKeyword {
            keywords: vec!["Created".to_string()],
        })
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<SearchBurrowResponse>().unwrap();
    assert!(res.found == 0 || res.burrows[0].burrow_id != created_burrow_id);

    // search post by tag
    let response = client
        .post("/search")
        .json(&json!(SearchRequest::SearchPostTag {
            tag: vec!["政治相关".to_string()]
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<SearchPostResponse>().unwrap();
    assert!(res.found == 0 || res.posts[0].post_id != post_id);

    // search post by keyword
    let response = client
        .post("/search")
        .json(&SearchRequest::SearchPostKeyword {
            keywords: vec!["test".to_string()],
        })
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<SearchMixResponse>().unwrap();
    assert!(res.replies.found == 0 || res.replies.replies[0].post_id != post_id);

    // Reopen the burrow with burrow_id
    let response = client
        .post("/admin")
        .json(&json!({ "ReopenBurrow": {"burrow_id": created_burrow_id} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    let response = client
        .post("/admin")
        .json(&json!({ "ReopenPost": {"post_id": post_id} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    let response = client
        .post("/admin")
        .json(&json!({ "ReopenReply": {"post_id": post_id, "reply_id": 0} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    std::thread::sleep(std::time::Duration::from_secs(1));

    // search burrow by keyword
    let response = client
        .post("/search")
        .json(&SearchRequest::SearchBurrowKeyword {
            keywords: vec!["Created".to_string()],
        })
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<SearchBurrowResponse>().unwrap();
    assert_eq!(res.burrows[0].burrow_id, created_burrow_id);

    // search post by tag
    let response = client
        .post("/search")
        .json(&json!(SearchRequest::SearchPostTag {
            tag: vec!["政治相关".to_string()]
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<SearchPostResponse>().unwrap();
    assert_eq!(res.posts[0].post_id, post_id);

    // search post by keyword
    let response = client
        .post("/search")
        .json(&SearchRequest::SearchPostKeyword {
            keywords: vec!["test".to_string()],
        })
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<SearchMixResponse>().unwrap();
    assert_eq!(res.replies.replies[0].post_id, post_id);

    // user log out
    let response = client
        .get("/users/logout")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    // ---------- Clean up ----------
    h2.abort();
    h3.abort();
    h4.abort();
    std::thread::sleep(std::time::Duration::from_secs(1));
    // ---------- Clean up ----------
}
