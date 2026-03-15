/// 其他语言的翻译（续）

use std::collections::HashMap;

/// 葡萄牙语翻译
pub fn pt_translations() -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    map.insert("app.name".to_string(), "ClawMesh".to_string());
    map.insert("app.subtitle".to_string(), "Sistema de gestão de comunidade inteligente".to_string());
    map.insert("app.description".to_string(), "Construído com Rust, fornecendo pontuação de crédito, gerenciamento de agentes e recursos de análise de dados".to_string());
    map.insert("app.version".to_string(), "v1.0.0".to_string());
    map.insert("app.powered_by".to_string(), "Powered by Rust 🦀".to_string());
    
    map.insert("nav.home".to_string(), "Início".to_string());
    map.insert("nav.back".to_string(), "Voltar ao início".to_string());
    map.insert("nav.credit".to_string(), "Sistema de crédito".to_string());
    map.insert("nav.agent".to_string(), "Gerenciamento de agentes".to_string());
    map.insert("nav.stats".to_string(), "Estatísticas".to_string());
    
    map.insert("home.welcome".to_string(), "Bem-vindo ao ClawMesh".to_string());
    map.insert("home.credit.title".to_string(), "Sistema de crédito".to_string());
    map.insert("home.credit.desc".to_string(), "Visualizar e gerenciar pontuações de crédito de usuários, rastrear mudanças de nível de reputação, monitorar a qualidade da comunidade em tempo real".to_string());
    map.insert("home.agent.title".to_string(), "Gerenciamento de agentes".to_string());
    map.insert("home.agent.desc".to_string(), "Gerenciar e monitorar agentes, visualizar status de batimento cardíaco, configurar tarefas automatizadas".to_string());
    map.insert("home.stats.title".to_string(), "Estatísticas".to_string());
    map.insert("home.stats.desc".to_string(), "Visualizar estatísticas globais, analisar comportamento do usuário, otimizar operações da comunidade".to_string());
    
    map.insert("credit.title".to_string(), "Sistema de crédito".to_string());
    map.insert("credit.score".to_string(), "Pontuação de crédito".to_string());
    map.insert("credit.tier".to_string(), "Nível de reputação".to_string());
    map.insert("credit.next_tier".to_string(), "Próximo nível".to_string());
    map.insert("credit.needed".to_string(), "Créditos necessários".to_string());
    map.insert("credit.rank".to_string(), "Classificação atual".to_string());
    
    map.insert("agent.title".to_string(), "Gerenciamento de agentes".to_string());
    map.insert("agent.total".to_string(), "Total de agentes".to_string());
    map.insert("agent.active".to_string(), "Agentes ativos".to_string());
    map.insert("agent.inactive".to_string(), "Agentes inativos".to_string());
    map.insert("agent.status.active".to_string(), "Ativo".to_string());
    map.insert("agent.status.inactive".to_string(), "Inativo".to_string());
    
    map.insert("stats.title".to_string(), "Estatísticas do sistema".to_string());
    map.insert("stats.total_users".to_string(), "Total de usuários".to_string());
    map.insert("stats.avg_credit".to_string(), "Crédito médio".to_string());
    map.insert("stats.growth".to_string(), "Crescimento mensal".to_string());
    
    map.insert("error.404.title".to_string(), "Página não encontrada".to_string());
    map.insert("error.404.message".to_string(), "Desculpe, a página que você procura não existe.\nPor favor, verifique a URL ou volte para a página inicial.".to_string());
    map.insert("error.500.title".to_string(), "Erro do servidor".to_string());
    map.insert("error.500.message".to_string(), "Desculpe, o servidor encontrou um erro.\nEstamos trabalhando para corrigi-lo. Por favor, tente novamente mais tarde.".to_string());
    
    map
}

/// 俄语翻译
pub fn ru_translations() -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    map.insert("app.name".to_string(), "ClawMesh".to_string());
    map.insert("app.subtitle".to_string(), "Интеллектуальная система управления сообществом".to_string());
    map.insert("app.description".to_string(), "Построено на Rust, предоставляет кредитный скоринг, управление агентами и функции анализа данных".to_string());
    map.insert("app.version".to_string(), "v1.0.0".to_string());
    map.insert("app.powered_by".to_string(), "Powered by Rust 🦀".to_string());
    
    map.insert("nav.home".to_string(), "Главная".to_string());
    map.insert("nav.back".to_string(), "Вернуться на главную".to_string());
    map.insert("nav.credit".to_string(), "Кредитная система".to_string());
    map.insert("nav.agent".to_string(), "Управление агентами".to_string());
    map.insert("nav.stats".to_string(), "Статистика".to_string());
    
    map.insert("home.welcome".to_string(), "Добро пожаловать в ClawMesh".to_string());
    map.insert("home.credit.title".to_string(), "Кредитная система".to_string());
    map.insert("home.credit.desc".to_string(), "Просмотр и управление кредитными баллами пользователей, отслеживание изменений уровня репутации, мониторинг качества сообщества в реальном времени".to_string());
    map.insert("home.agent.title".to_string(), "Управление агентами".to_string());
    map.insert("home.agent.desc".to_string(), "Управление и мониторинг агентов, просмотр состояния пульса, настройка автоматизированных задач".to_string());
    map.insert("home.stats.title".to_string(), "Статистика".to_string());
    map.insert("home.stats.desc".to_string(), "Просмотр глобальной статистики, анализ поведения пользователей, оптимизация операций сообщества".to_string());
    
    map.insert("credit.title".to_string(), "Кредитная система".to_string());
    map.insert("credit.score".to_string(), "Кредитный балл".to_string());
    map.insert("credit.tier".to_string(), "Уровень репутации".to_string());
    map.insert("credit.next_tier".to_string(), "Следующий уровень".to_string());
    map.insert("credit.needed".to_string(), "Необходимо кредитов".to_string());
    map.insert("credit.rank".to_string(), "Текущий ранг".to_string());
    
    map.insert("agent.title".to_string(), "Управление агентами".to_string());
    map.insert("agent.total".to_string(), "Всего агентов".to_string());
    map.insert("agent.active".to_string(), "Активные агенты".to_string());
    map.insert("agent.inactive".to_string(), "Неактивные агенты".to_string());
    map.insert("agent.status.active".to_string(), "Активен".to_string());
    map.insert("agent.status.inactive".to_string(), "Неактивен".to_string());
    
    map.insert("stats.title".to_string(), "Системная статистика".to_string());
    map.insert("stats.total_users".to_string(), "Всего пользователей".to_string());
    map.insert("stats.avg_credit".to_string(), "Средний кредит".to_string());
    map.insert("stats.growth".to_string(), "Месячный рост".to_string());
    
    map.insert("error.404.title".to_string(), "Страница не найдена".to_string());
    map.insert("error.404.message".to_string(), "Извините, страница, которую вы ищете, не существует.\nПожалуйста, проверьте URL или вернитесь на главную страницу.".to_string());
    map.insert("error.500.title".to_string(), "Ошибка сервера".to_string());
    map.insert("error.500.message".to_string(), "Извините, сервер столкнулся с ошибкой.\nМы работаем над ее устранением. Пожалуйста, попробуйте позже.".to_string());
    
    map
}

/// 阿拉伯语翻译
pub fn ar_translations() -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    map.insert("app.name".to_string(), "ClawMesh".to_string());
    map.insert("app.subtitle".to_string(), "نظام إدارة المجتمع الذكي".to_string());
    map.insert("app.description".to_string(), "مبني بـ Rust، يوفر تقييم الائتمان وإدارة الوكلاء وميزات تحليل البيانات".to_string());
    map.insert("app.version".to_string(), "v1.0.0".to_string());
    map.insert("app.powered_by".to_string(), "Powered by Rust 🦀".to_string());
    
    map.insert("nav.home".to_string(), "الرئيسية".to_string());
    map.insert("nav.back".to_string(), "العودة إلى الرئيسية".to_string());
    map.insert("nav.credit".to_string(), "نظام الائتمان".to_string());
    map.insert("nav.agent".to_string(), "إدارة الوكلاء".to_string());
    map.insert("nav.stats".to_string(), "الإحصائيات".to_string());
    
    map.insert("home.welcome".to_string(), "مرحبًا بك في ClawMesh".to_string());
    map.insert("home.credit.title".to_string(), "نظام الائتمان".to_string());
    map.insert("home.credit.desc".to_string(), "عرض وإدارة درجات ائتمان المستخدمين، تتبع تغييرات مستوى السمعة، مراقبة جودة المجتمع في الوقت الفعلي".to_string());
    map.insert("home.agent.title".to_string(), "إدارة الوكلاء".to_string());
    map.insert("home.agent.desc".to_string(), "إدارة ومراقبة الوكلاء، عرض حالة نبضات القلب، تكوين المهام الآلية".to_string());
    map.insert("home.stats.title".to_string(), "الإحصائيات".to_string());
    map.insert("home.stats.desc".to_string(), "عرض الإحصائيات العالمية، تحليل سلوك المستخدم، تحسين عمليات المجتمع".to_string());
    
    map.insert("credit.title".to_string(), "نظام الائتمان".to_string());
    map.insert("credit.score".to_string(), "درجة الائتمان".to_string());
    map.insert("credit.tier".to_string(), "مستوى السمعة".to_string());
    map.insert("credit.next_tier".to_string(), "المستوى التالي".to_string());
    map.insert("credit.needed".to_string(), "الاعتمادات المطلوبة".to_string());
    map.insert("credit.rank".to_string(), "الترتيب الحالي".to_string());
    
    map.insert("agent.title".to_string(), "إدارة الوكلاء".to_string());
    map.insert("agent.total".to_string(), "إجمالي الوكلاء".to_string());
    map.insert("agent.active".to_string(), "الوكلاء النشطون".to_string());
    map.insert("agent.inactive".to_string(), "الوكلاء غير النشطين".to_string());
    map.insert("agent.status.active".to_string(), "نشط".to_string());
    map.insert("agent.status.inactive".to_string(), "غير نشط".to_string());
    
    map.insert("stats.title".to_string(), "إحصائيات النظام".to_string());
    map.insert("stats.total_users".to_string(), "إجمالي المستخدمين".to_string());
    map.insert("stats.avg_credit".to_string(), "متوسط الائتمان".to_string());
    map.insert("stats.growth".to_string(), "النمو الشهري".to_string());
    
    map.insert("error.404.title".to_string(), "الصفحة غير موجودة".to_string());
    map.insert("error.404.message".to_string(), "عذرًا، الصفحة التي تبحث عنها غير موجودة.\nيرجى التحقق من عنوان URL أو العودة إلى الصفحة الرئيسية.".to_string());
    map.insert("error.500.title".to_string(), "خطأ في الخادم".to_string());
    map.insert("error.500.message".to_string(), "عذرًا، واجه الخادم خطأ.\nنحن نعمل على إصلاحه. يرجى المحاولة مرة أخرى لاحقًا.".to_string());
    
    map
}

/// 印地语翻译
pub fn hi_translations() -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    map.insert("app.name".to_string(), "ClawMesh".to_string());
    map.insert("app.subtitle".to_string(), "बुद्धिमान समुदाय प्रबंधन प्रणाली".to_string());
    map.insert("app.description".to_string(), "Rust के साथ निर्मित, क्रेडिट स्कोरिंग, एजेंट प्रबंधन और डेटा विश्लेषण सुविधाएं प्रदान करता है".to_string());
    map.insert("app.version".to_string(), "v1.0.0".to_string());
    map.insert("app.powered_by".to_string(), "Powered by Rust 🦀".to_string());
    
    map.insert("nav.home".to_string(), "होम".to_string());
    map.insert("nav.back".to_string(), "होम पर वापस जाएं".to_string());
    map.insert("nav.credit".to_string(), "क्रेडिट प्रणाली".to_string());
    map.insert("nav.agent".to_string(), "एजेंट प्रबंधन".to_string());
    map.insert("nav.stats".to_string(), "आंकड़े".to_string());
    
    map.insert("home.welcome".to_string(), "ClawMesh में आपका स्वागत है".to_string());
    map.insert("home.credit.title".to_string(), "क्रेडिट प्रणाली".to_string());
    map.insert("home.credit.desc".to_string(), "उपयोगकर्ता क्रेडिट स्कोर देखें और प्रबंधित करें, प्रतिष्ठा स्तर परिवर्तनों को ट्रैक करें, वास्तविक समय में समुदाय गुणवत्ता की निगरानी करें".to_string());
    map.insert("home.agent.title".to_string(), "एजेंट प्रबंधन".to_string());
    map.insert("home.agent.desc".to_string(), "एजेंटों का प्रबंधन और निगरानी करें, हार्टबीट स्थिति देखें, स्वचालित कार्य कॉन्फ़िगर करें".to_string());
    map.insert("home.stats.title".to_string(), "आंकड़े".to_string());
    map.insert("home.stats.desc".to_string(), "वैश्विक आंकड़े देखें, उपयोगकर्ता व्यवहार का विश्लेषण करें, समुदाय संचालन को अनुकूलित करें".to_string());
    
    map.insert("credit.title".to_string(), "क्रेडिट प्रणाली".to_string());
    map.insert("credit.score".to_string(), "क्रेडिट स्कोर".to_string());
    map.insert("credit.tier".to_string(), "प्रतिष्ठा स्तर".to_string());
    map.insert("credit.next_tier".to_string(), "अगला स्तर".to_string());
    map.insert("credit.needed".to_string(), "आवश्यक क्रेडिट".to_string());
    map.insert("credit.rank".to_string(), "वर्तमान रैंक".to_string());
    
    map.insert("agent.title".to_string(), "एजेंट प्रबंधन".to_string());
    map.insert("agent.total".to_string(), "कुल एजेंट".to_string());
    map.insert("agent.active".to_string(), "सक्रिय एजेंट".to_string());
    map.insert("agent.inactive".to_string(), "निष्क्रिय एजेंट".to_string());
    map.insert("agent.status.active".to_string(), "सक्रिय".to_string());
    map.insert("agent.status.inactive".to_string(), "निष्क्रिय".to_string());
    
    map.insert("stats.title".to_string(), "सिस्टम आंकड़े".to_string());
    map.insert("stats.total_users".to_string(), "कुल उपयोगकर्ता".to_string());
    map.insert("stats.avg_credit".to_string(), "औसत क्रेडिट".to_string());
    map.insert("stats.growth".to_string(), "मासिक वृद्धि".to_string());
    
    map.insert("error.404.title".to_string(), "पृष्ठ नहीं मिला".to_string());
    map.insert("error.404.message".to_string(), "क्षमा करें, आप जो पृष्ठ खोज रहे हैं वह मौजूद नहीं है।\nकृपया URL जांचें या होम पेज पर वापस जाएं।".to_string());
    map.insert("error.500.title".to_string(), "सर्वर त्रुटि".to_string());
    map.insert("error.500.message".to_string(), "क्षमा करें, सर्वर में एक त्रुटि आई है।\nहम इसे ठीक करने पर काम कर रहे हैं। कृपया बाद में पुनः प्रयास करें।".to_string());
    
    map
}

/// 意大利语翻译
pub fn it_translations() -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    map.insert("app.name".to_string(), "ClawMesh".to_string());
    map.insert("app.subtitle".to_string(), "Sistema di gestione della comunità intelligente".to_string());
    map.insert("app.description".to_string(), "Costruito con Rust, fornisce punteggio di credito, gestione degli agenti e funzionalità di analisi dei dati".to_string());
    map.insert("app.version".to_string(), "v1.0.0".to_string());
    map.insert("app.powered_by".to_string(), "Powered by Rust 🦀".to_string());
    
    map.insert("nav.home".to_string(), "Home".to_string());
    map.insert("nav.back".to_string(), "Torna alla home".to_string());
    map.insert("nav.credit".to_string(), "Sistema di credito".to_string());
    map.insert("nav.agent".to_string(), "Gestione agenti".to_string());
    map.insert("nav.stats".to_string(), "Statistiche".to_string());
    
    map.insert("home.welcome".to_string(), "Benvenuto su ClawMesh".to_string());
    map.insert("home.credit.title".to_string(), "Sistema di credito".to_string());
    map.insert("home.credit.desc".to_string(), "Visualizza e gestisci i punteggi di credito degli utenti, traccia i cambiamenti del livello di reputazione, monitora la qualità della comunità in tempo reale".to_string());
    map.insert("home.agent.title".to_string(), "Gestione agenti".to_string());
    map.insert("home.agent.desc".to_string(), "Gestisci e monitora gli agenti, visualizza lo stato del battito cardiaco, configura le attività automatizzate".to_string());
    map.insert("home.stats.title".to_string(), "Statistiche".to_string());
    map.insert("home.stats.desc".to_string(), "Visualizza statistiche globali, analizza il comportamento degli utenti, ottimizza le operazioni della comunità".to_string());
    
    map.insert("credit.title".to_string(), "Sistema di credito".to_string());
    map.insert("credit.score".to_string(), "Punteggio di credito".to_string());
    map.insert("credit.tier".to_string(), "Livello di reputazione".to_string());
    map.insert("credit.next_tier".to_string(), "Livello successivo".to_string());
    map.insert("credit.needed".to_string(), "Crediti necessari".to_string());
    map.insert("credit.rank".to_string(), "Classifica attuale".to_string());
    
    map.insert("agent.title".to_string(), "Gestione agenti".to_string());
    map.insert("agent.total".to_string(), "Totale agenti".to_string());
    map.insert("agent.active".to_string(), "Agenti attivi".to_string());
    map.insert("agent.inactive".to_string(), "Agenti inattivi".to_string());
    map.insert("agent.status.active".to_string(), "Attivo".to_string());
    map.insert("agent.status.inactive".to_string(), "Inattivo".to_string());
    
    map.insert("stats.title".to_string(), "Statistiche di sistema".to_string());
    map.insert("stats.total_users".to_string(), "Totale utenti".to_string());
    map.insert("stats.avg_credit".to_string(), "Credito medio".to_string());
    map.insert("stats.growth".to_string(), "Crescita mensile".to_string());
    
    map.insert("error.404.title".to_string(), "Pagina non trovata".to_string());
    map.insert("error.404.message".to_string(), "Spiacenti, la pagina che stai cercando non esiste.\nSi prega di controllare l'URL o tornare alla home page.".to_string());
    map.insert("error.500.title".to_string(), "Errore del server".to_string());
    map.insert("error.500.message".to_string(), "Spiacenti, il server ha riscontrato un errore.\nStiamo lavorando per risolverlo. Si prega di riprovare più tardi.".to_string());
    
    map
}

// 继续其他语言...
pub fn nl_translations() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("app.name".to_string(), "ClawMesh".to_string());
    map.insert("app.subtitle".to_string(), "Intelligent Community Management Systeem".to_string());
    map.insert("nav.home".to_string(), "Home".to_string());
    map.insert("nav.credit".to_string(), "Creditsysteem".to_string());
    map.insert("home.welcome".to_string(), "Welkom bij ClawMesh".to_string());
    map.insert("credit.title".to_string(), "Creditsysteem".to_string());
    map.insert("error.404.title".to_string(), "Pagina niet gevonden".to_string());
    map
}

pub fn tr_translations() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("app.name".to_string(), "ClawMesh".to_string());
    map.insert("app.subtitle".to_string(), "Akıllı Topluluk Yönetim Sistemi".to_string());
    map.insert("nav.home".to_string(), "Ana Sayfa".to_string());
    map.insert("nav.credit".to_string(), "Kredi Sistemi".to_string());
    map.insert("home.welcome".to_string(), "ClawMesh'e Hoş Geldiniz".to_string());
    map.insert("credit.title".to_string(), "Kredi Sistemi".to_string());
    map.insert("error.404.title".to_string(), "Sayfa Bulunamadı".to_string());
    map
}

pub fn pl_translations() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("app.name".to_string(), "ClawMesh".to_string());
    map.insert("app.subtitle".to_string(), "Inteligentny System Zarządzania Społecznością".to_string());
    map.insert("nav.home".to_string(), "Strona główna".to_string());
    map.insert("nav.credit".to_string(), "System kredytowy".to_string());
    map.insert("home.welcome".to_string(), "Witamy w ClawMesh".to_string());
    map.insert("credit.title".to_string(), "System kredytowy".to_string());
    map.insert("error.404.title".to_string(), "Strona nie znaleziona".to_string());
    map
}

pub fn vi_translations() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("app.name".to_string(), "ClawMesh".to_string());
    map.insert("app.subtitle".to_string(), "Hệ thống Quản lý Cộng đồng Thông minh".to_string());
    map.insert("nav.home".to_string(), "Trang chủ".to_string());
    map.insert("nav.credit".to_string(), "Hệ thống tín dụng".to_string());
    map.insert("home.welcome".to_string(), "Chào mừng đến với ClawMesh".to_string());
    map.insert("credit.title".to_string(), "Hệ thống tín dụng".to_string());
    map.insert("error.404.title".to_string(), "Không tìm thấy trang".to_string());
    map
}
