use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")] // 自动处理 Liquibase 常用的驼峰命名
pub struct MasterChangeLog {
    // 使用 rename 处理标签名，并允许忽略其他属性
    #[serde(rename = "include", default)]
    pub includes: Vec<Include>,
}

#[derive(Debug, Deserialize)]
pub struct Include {
    // 显式指定从属性中读取 file
    #[serde(rename = "@file")]
    pub file: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChildChangeLog {
    #[serde(rename = "changeSet", default)]
    pub change_sets: Vec<ChangeSet>,
}

#[derive(Debug, Deserialize)]
pub struct ChangeSet {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@author")]
    pub author: String,
    // 提取 <sql> 标签内的文本内容
    pub sql: String,
}
