/// 所有语言的翻译（14种新增语言）
/// 
/// 使用 AI 翻译生成，可能需要母语者校对

use std::collections::HashMap;

/// 日语翻译
pub fn ja_translations() -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    map.insert("app.name".to_string(), "ClawMesh".to_string());
    map.insert("app.subtitle".to_string(), "インテリジェントコミュニティ管理システム".to_string());
    map.insert("app.description".to_string(), "Rustで構築され、信用スコアリング、エージェント管理、データ分析機能を提供".to_string());
    map.insert("app.version".to_string(), "v1.0.0".to_string());
    map.insert("app.powered_by".to_string(), "Powered by Rust 🦀".to_string());
    
    map.insert("nav.home".to_string(), "ホーム".to_string());
    map.insert("nav.back".to_string(), "ホームに戻る".to_string());
    map.insert("nav.credit".to_string(), "信用システム".to_string());
    map.insert("nav.agent".to_string(), "エージェント管理".to_string());
    map.insert("nav.stats".to_string(), "統計".to_string());
    
    map.insert("home.welcome".to_string(), "ClawMeshへようこそ".to_string());
    map.insert("home.credit.title".to_string(), "信用システム".to_string());
    map.insert("home.credit.desc".to_string(), "ユーザーの信用スコアを表示・管理し、評判レベルの変化を追跡し、コミュニティの品質をリアルタイムで監視".to_string());
    map.insert("home.agent.title".to_string(), "エージェント管理".to_string());
    map.insert("home.agent.desc".to_string(), "エージェントの管理と監視、ハートビートステータスの表示、自動化タスクの設定".to_string());
    map.insert("home.stats.title".to_string(), "統計".to_string());
    map.insert("home.stats.desc".to_string(), "グローバル統計の表示、ユーザー行動の分析、コミュニティ運営の最適化".to_string());
    
    map.insert("credit.title".to_string(), "信用システム".to_string());
    map.insert("credit.score".to_string(), "信用スコア".to_string());
    map.insert("credit.tier".to_string(), "評判レベル".to_string());
    map.insert("credit.next_tier".to_string(), "次のレベル".to_string());
    map.insert("credit.needed".to_string(), "必要な信用".to_string());
    map.insert("credit.rank".to_string(), "現在のランク".to_string());
    
    map.insert("agent.title".to_string(), "エージェント管理".to_string());
    map.insert("agent.total".to_string(), "総エージェント数".to_string());
    map.insert("agent.active".to_string(), "アクティブなエージェント".to_string());
    map.insert("agent.inactive".to_string(), "非アクティブなエージェント".to_string());
    map.insert("agent.status.active".to_string(), "アクティブ".to_string());
    map.insert("agent.status.inactive".to_string(), "非アクティブ".to_string());
    
    map.insert("stats.title".to_string(), "システム統計".to_string());
    map.insert("stats.total_users".to_string(), "総ユーザー数".to_string());
    map.insert("stats.avg_credit".to_string(), "平均信用".to_string());
    map.insert("stats.growth".to_string(), "月間成長".to_string());
    
    map.insert("error.404.title".to_string(), "ページが見つかりません".to_string());
    map.insert("error.404.message".to_string(), "申し訳ございません。お探しのページは存在しません。\nURLを確認するか、ホームページに戻ってください。".to_string());
    map.insert("error.500.title".to_string(), "サーバーエラー".to_string());
    map.insert("error.500.message".to_string(), "申し訳ございません。サーバーでエラーが発生しました。\n修正中ですので、後でもう一度お試しください。".to_string());
    
    map
}

/// 韩语翻译
pub fn ko_translations() -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    map.insert("app.name".to_string(), "ClawMesh".to_string());
    map.insert("app.subtitle".to_string(), "지능형 커뮤니티 관리 시스템".to_string());
    map.insert("app.description".to_string(), "Rust로 구축되어 신용 점수, 에이전트 관리 및 데이터 분석 기능 제공".to_string());
    map.insert("app.version".to_string(), "v1.0.0".to_string());
    map.insert("app.powered_by".to_string(), "Powered by Rust 🦀".to_string());
    
    map.insert("nav.home".to_string(), "홈".to_string());
    map.insert("nav.back".to_string(), "홈으로 돌아가기".to_string());
    map.insert("nav.credit".to_string(), "신용 시스템".to_string());
    map.insert("nav.agent".to_string(), "에이전트 관리".to_string());
    map.insert("nav.stats".to_string(), "통계".to_string());
    
    map.insert("home.welcome".to_string(), "ClawMesh에 오신 것을 환영합니다".to_string());
    map.insert("home.credit.title".to_string(), "신용 시스템".to_string());
    map.insert("home.credit.desc".to_string(), "사용자 신용 점수 보기 및 관리, 평판 등급 변화 추적, 실시간 커뮤니티 품질 모니터링".to_string());
    map.insert("home.agent.title".to_string(), "에이전트 관리".to_string());
    map.insert("home.agent.desc".to_string(), "에이전트 관리 및 모니터링, 하트비트 상태 보기, 자동화 작업 구성".to_string());
    map.insert("home.stats.title".to_string(), "통계".to_string());
    map.insert("home.stats.desc".to_string(), "글로벌 통계 보기, 사용자 행동 분석, 커뮤니티 운영 최적화".to_string());
    
    map.insert("credit.title".to_string(), "신용 시스템".to_string());
    map.insert("credit.score".to_string(), "신용 점수".to_string());
    map.insert("credit.tier".to_string(), "평판 등급".to_string());
    map.insert("credit.next_tier".to_string(), "다음 등급".to_string());
    map.insert("credit.needed".to_string(), "필요한 크레딧".to_string());
    map.insert("credit.rank".to_string(), "현재 순위".to_string());
    
    map.insert("agent.title".to_string(), "에이전트 관리".to_string());
    map.insert("agent.total".to_string(), "총 에이전트".to_string());
    map.insert("agent.active".to_string(), "활성 에이전트".to_string());
    map.insert("agent.inactive".to_string(), "비활성 에이전트".to_string());
    map.insert("agent.status.active".to_string(), "활성".to_string());
    map.insert("agent.status.inactive".to_string(), "비활성".to_string());
    
    map.insert("stats.title".to_string(), "시스템 통계".to_string());
    map.insert("stats.total_users".to_string(), "총 사용자".to_string());
    map.insert("stats.avg_credit".to_string(), "평균 신용".to_string());
    map.insert("stats.growth".to_string(), "월간 성장".to_string());
    
    map.insert("error.404.title".to_string(), "페이지를 찾을 수 없습니다".to_string());
    map.insert("error.404.message".to_string(), "죄송합니다. 찾으시는 페이지가 존재하지 않습니다.\nURL을 확인하거나 홈페이지로 돌아가세요.".to_string());
    map.insert("error.500.title".to_string(), "서버 오류".to_string());
    map.insert("error.500.message".to_string(), "죄송합니다. 서버에서 오류가 발생했습니다.\n수정 중이니 나중에 다시 시도해 주세요.".to_string());
    
    map
}

/// 法语翻译
pub fn fr_translations() -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    map.insert("app.name".to_string(), "ClawMesh".to_string());
    map.insert("app.subtitle".to_string(), "Système de gestion de communauté intelligente".to_string());
    map.insert("app.description".to_string(), "Construit avec Rust, offrant notation de crédit, gestion d'agents et analyse de données".to_string());
    map.insert("app.version".to_string(), "v1.0.0".to_string());
    map.insert("app.powered_by".to_string(), "Propulsé par Rust 🦀".to_string());
    
    map.insert("nav.home".to_string(), "Accueil".to_string());
    map.insert("nav.back".to_string(), "Retour à l'accueil".to_string());
    map.insert("nav.credit".to_string(), "Système de crédit".to_string());
    map.insert("nav.agent".to_string(), "Gestion des agents".to_string());
    map.insert("nav.stats".to_string(), "Statistiques".to_string());
    
    map.insert("home.welcome".to_string(), "Bienvenue sur ClawMesh".to_string());
    map.insert("home.credit.title".to_string(), "Système de crédit".to_string());
    map.insert("home.credit.desc".to_string(), "Afficher et gérer les scores de crédit des utilisateurs, suivre les changements de niveau de réputation, surveiller la qualité de la communauté en temps réel".to_string());
    map.insert("home.agent.title".to_string(), "Gestion des agents".to_string());
    map.insert("home.agent.desc".to_string(), "Gérer et surveiller les agents, afficher l'état des battements de cœur, configurer les tâches automatisées".to_string());
    map.insert("home.stats.title".to_string(), "Statistiques".to_string());
    map.insert("home.stats.desc".to_string(), "Afficher les statistiques globales, analyser le comportement des utilisateurs, optimiser les opérations communautaires".to_string());
    
    map.insert("credit.title".to_string(), "Système de crédit".to_string());
    map.insert("credit.score".to_string(), "Score de crédit".to_string());
    map.insert("credit.tier".to_string(), "Niveau de réputation".to_string());
    map.insert("credit.next_tier".to_string(), "Niveau suivant".to_string());
    map.insert("credit.needed".to_string(), "Crédits nécessaires".to_string());
    map.insert("credit.rank".to_string(), "Rang actuel".to_string());
    
    map.insert("agent.title".to_string(), "Gestion des agents".to_string());
    map.insert("agent.total".to_string(), "Total des agents".to_string());
    map.insert("agent.active".to_string(), "Agents actifs".to_string());
    map.insert("agent.inactive".to_string(), "Agents inactifs".to_string());
    map.insert("agent.status.active".to_string(), "Actif".to_string());
    map.insert("agent.status.inactive".to_string(), "Inactif".to_string());
    
    map.insert("stats.title".to_string(), "Statistiques du système".to_string());
    map.insert("stats.total_users".to_string(), "Total des utilisateurs".to_string());
    map.insert("stats.avg_credit".to_string(), "Crédit moyen".to_string());
    map.insert("stats.growth".to_string(), "Croissance mensuelle".to_string());
    
    map.insert("error.404.title".to_string(), "Page non trouvée".to_string());
    map.insert("error.404.message".to_string(), "Désolé, la page que vous recherchez n'existe pas.\nVeuillez vérifier l'URL ou retourner à la page d'accueil.".to_string());
    map.insert("error.500.title".to_string(), "Erreur du serveur".to_string());
    map.insert("error.500.message".to_string(), "Désolé, le serveur a rencontré une erreur.\nNous travaillons à la résoudre. Veuillez réessayer plus tard.".to_string());
    
    map
}

/// 德语翻译
pub fn de_translations() -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    map.insert("app.name".to_string(), "ClawMesh".to_string());
    map.insert("app.subtitle".to_string(), "Intelligentes Community-Management-System".to_string());
    map.insert("app.description".to_string(), "Mit Rust erstellt, bietet Kreditbewertung, Agentenverwaltung und Datenanalysefunktionen".to_string());
    map.insert("app.version".to_string(), "v1.0.0".to_string());
    map.insert("app.powered_by".to_string(), "Powered by Rust 🦀".to_string());
    
    map.insert("nav.home".to_string(), "Startseite".to_string());
    map.insert("nav.back".to_string(), "Zurück zur Startseite".to_string());
    map.insert("nav.credit".to_string(), "Kreditsystem".to_string());
    map.insert("nav.agent".to_string(), "Agentenverwaltung".to_string());
    map.insert("nav.stats".to_string(), "Statistiken".to_string());
    
    map.insert("home.welcome".to_string(), "Willkommen bei ClawMesh".to_string());
    map.insert("home.credit.title".to_string(), "Kreditsystem".to_string());
    map.insert("home.credit.desc".to_string(), "Kreditscores von Benutzern anzeigen und verwalten, Änderungen der Reputationsstufe verfolgen, Community-Qualität in Echtzeit überwachen".to_string());
    map.insert("home.agent.title".to_string(), "Agentenverwaltung".to_string());
    map.insert("home.agent.desc".to_string(), "Agenten verwalten und überwachen, Heartbeat-Status anzeigen, automatisierte Aufgaben konfigurieren".to_string());
    map.insert("home.stats.title".to_string(), "Statistiken".to_string());
    map.insert("home.stats.desc".to_string(), "Globale Statistiken anzeigen, Benutzerverhalten analysieren, Community-Betrieb optimieren".to_string());
    
    map.insert("credit.title".to_string(), "Kreditsystem".to_string());
    map.insert("credit.score".to_string(), "Kreditscore".to_string());
    map.insert("credit.tier".to_string(), "Reputationsstufe".to_string());
    map.insert("credit.next_tier".to_string(), "Nächste Stufe".to_string());
    map.insert("credit.needed".to_string(), "Benötigte Credits".to_string());
    map.insert("credit.rank".to_string(), "Aktueller Rang".to_string());
    
    map.insert("agent.title".to_string(), "Agentenverwaltung".to_string());
    map.insert("agent.total".to_string(), "Gesamtzahl der Agenten".to_string());
    map.insert("agent.active".to_string(), "Aktive Agenten".to_string());
    map.insert("agent.inactive".to_string(), "Inaktive Agenten".to_string());
    map.insert("agent.status.active".to_string(), "Aktiv".to_string());
    map.insert("agent.status.inactive".to_string(), "Inaktiv".to_string());
    
    map.insert("stats.title".to_string(), "Systemstatistiken".to_string());
    map.insert("stats.total_users".to_string(), "Gesamtbenutzer".to_string());
    map.insert("stats.avg_credit".to_string(), "Durchschnittlicher Kredit".to_string());
    map.insert("stats.growth".to_string(), "Monatliches Wachstum".to_string());
    
    map.insert("error.404.title".to_string(), "Seite nicht gefunden".to_string());
    map.insert("error.404.message".to_string(), "Entschuldigung, die gesuchte Seite existiert nicht.\nBitte überprüfen Sie die URL oder kehren Sie zur Startseite zurück.".to_string());
    map.insert("error.500.title".to_string(), "Serverfehler".to_string());
    map.insert("error.500.message".to_string(), "Entschuldigung, der Server hat einen Fehler festgestellt.\nWir arbeiten daran. Bitte versuchen Sie es später erneut.".to_string());
    
    map
}

/// 西班牙语翻译
pub fn es_translations() -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    map.insert("app.name".to_string(), "ClawMesh".to_string());
    map.insert("app.subtitle".to_string(), "Sistema de gestión de comunidad inteligente".to_string());
    map.insert("app.description".to_string(), "Construido con Rust, proporciona calificación crediticia, gestión de agentes y análisis de datos".to_string());
    map.insert("app.version".to_string(), "v1.0.0".to_string());
    map.insert("app.powered_by".to_string(), "Powered by Rust 🦀".to_string());
    
    map.insert("nav.home".to_string(), "Inicio".to_string());
    map.insert("nav.back".to_string(), "Volver al inicio".to_string());
    map.insert("nav.credit".to_string(), "Sistema de crédito".to_string());
    map.insert("nav.agent".to_string(), "Gestión de agentes".to_string());
    map.insert("nav.stats".to_string(), "Estadísticas".to_string());
    
    map.insert("home.welcome".to_string(), "Bienvenido a ClawMesh".to_string());
    map.insert("home.credit.title".to_string(), "Sistema de crédito".to_string());
    map.insert("home.credit.desc".to_string(), "Ver y gestionar puntuaciones de crédito de usuarios, rastrear cambios de nivel de reputación, monitorear la calidad de la comunidad en tiempo real".to_string());
    map.insert("home.agent.title".to_string(), "Gestión de agentes".to_string());
    map.insert("home.agent.desc".to_string(), "Gestionar y monitorear agentes, ver estado de latidos, configurar tareas automatizadas".to_string());
    map.insert("home.stats.title".to_string(), "Estadísticas".to_string());
    map.insert("home.stats.desc".to_string(), "Ver estadísticas globales, analizar comportamiento de usuarios, optimizar operaciones comunitarias".to_string());
    
    map.insert("credit.title".to_string(), "Sistema de crédito".to_string());
    map.insert("credit.score".to_string(), "Puntuación de crédito".to_string());
    map.insert("credit.tier".to_string(), "Nivel de reputación".to_string());
    map.insert("credit.next_tier".to_string(), "Siguiente nivel".to_string());
    map.insert("credit.needed".to_string(), "Créditos necesarios".to_string());
    map.insert("credit.rank".to_string(), "Rango actual".to_string());
    
    map.insert("agent.title".to_string(), "Gestión de agentes".to_string());
    map.insert("agent.total".to_string(), "Total de agentes".to_string());
    map.insert("agent.active".to_string(), "Agentes activos".to_string());
    map.insert("agent.inactive".to_string(), "Agentes inactivos".to_string());
    map.insert("agent.status.active".to_string(), "Activo".to_string());
    map.insert("agent.status.inactive".to_string(), "Inactivo".to_string());
    
    map.insert("stats.title".to_string(), "Estadísticas del sistema".to_string());
    map.insert("stats.total_users".to_string(), "Total de usuarios".to_string());
    map.insert("stats.avg_credit".to_string(), "Crédito promedio".to_string());
    map.insert("stats.growth".to_string(), "Crecimiento mensual".to_string());
    
    map.insert("error.404.title".to_string(), "Página no encontrada".to_string());
    map.insert("error.404.message".to_string(), "Lo sentimos, la página que busca no existe.\nPor favor verifique la URL o vuelva a la página de inicio.".to_string());
    map.insert("error.500.title".to_string(), "Error del servidor".to_string());
    map.insert("error.500.message".to_string(), "Lo sentimos, el servidor encontró un error.\nEstamos trabajando en solucionarlo. Por favor intente más tarde.".to_string());
    
    map
}

// 由于代码太长，我将在下一个文件中继续添加其他语言的翻译
