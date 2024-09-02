<template>
    <div>
        <el-select v-model="selectedTable" placeholder="Select a table" @change="fetchTableData">
            <el-option v-for="table in tables" :key="table" :label="table" :value="table" />
        </el-select>
        <el-table :data="tableData" v-if="tableData.length > 0">
            <el-table-column v-for="(value, key) in tableData[0] || {}" :key="key" :prop="key" :label="key" />
        </el-table>
        <p v-else>No data to display</p>
    </div>
</template>

<script lang="ts" setup>
import { ref, onMounted } from 'vue';
import Database from '@tauri-apps/plugin-sql';

let db: any; // 声明数据库连接变量

const tables = ref<string[]>([]); // 初始为空数组
const selectedTable = ref<string>('');
const tableData = ref<Array<Record<string, any>>>([]); // 初始化为一个空数组

const loadDatabase = () => {
    Database.load('sqlite:bintool.db') // 替换为实际的数据库路径
        .then(database => {
            db = database;
            return fetchTables(); // 获取表格列表
        })
        .catch(error => {
            console.error("Error loading database:", error);
        });
};

const fetchTables = async () => {
    try {
        const result = await db.select("SELECT name FROM sqlite_master WHERE type='table';");
        tables.value = result.map((row: { name: string }) => row.name);
    } catch (error) {
        console.error("Error fetching tables:", error);
    }
};

const fetchTableData = async (tableName: string) => {
    if (!tableName) return;
    try {
        const data = await db.select(`SELECT * FROM ${tableName};`);
        tableData.value = data;
    } catch (error) {
        console.error("Error fetching table data:", error);
    }
};

onMounted(() => {
    loadDatabase();
});
</script>

<style scoped></style>
