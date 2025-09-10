const axios = require('axios');
const fs = require('fs').promises;
const path = require('path');

// 抖音直播间数据获取脚本
// 使用Node.js实现curl命令功能

async function fetchDouyinData() {
    console.log('正在获取抖音直播间数据...');
    
    try {
        // 配置URL参数
        const baseUrl = 'https://live.douyin.com/webcast/room/web/enter/';
        const params = {
            aid: '6383',
            app_name: 'douyin_web',
            live_id: '1',
            device_platform: 'web',
            language: 'zh-CN',
            enter_from: 'web_homepage_follow',
            cookie_enabled: 'true',
            screen_width: '1920',
            screen_height: '1080',
            browser_language: 'zh-CN',
            browser_platform: 'MacIntel',
            browser_name: 'Chrome',
            browser_version: '140.0.0.0',
            web_rid: '671728942628',
            enter_source: '',
            is_need_double_stream: 'false',
            insert_task_id: '',
            live_reason: '',
            msToken: 'djIQSLNfdq3BLVY9-hIFbpJVQs238wUtsl1_Zvc2-rkmUSUy44JUt-L_jMcpo--kcwpK8Sc4C7fUvX-QL-mrqE1RM0E65tIZ8Rz4UoVXrzbCAhvwNKSX0TG8r1KNdI3K9dbBvI3Lb6W62nr7LStyw-41pkfZkFW2Vfi9zqnnLDSM-NMhCJTrxQ%3D%3D',
            a_bogus: 'EJ0fkF67Dx%2FfPdKGuObyCHlU2lxMNB8yQZixWCluCNzJOXUTjuP7gcbZboqs4doR3bpsiHIHTx0lYEncTdUs1ZrkumkfSmzyJzACVgsL8qwsGFJQgHfZeukFqwBN0Rsqa%2FcIE1g78sBK2d5W9HAQldBaC5Pa5QmDWHqydM9bj9WbDAyPu3rROMEWiEwPBQ2-rf%3D%3D'
        };
        
        // 构建完整URL
        const queryString = Object.entries(params)
            .map(([key, value]) => `${key}=${value}`)
            .join('&');
        const url = `${baseUrl}?${queryString}`;
        
        // 配置请求头（对应shell脚本中的-H参数）
        const headers = {
            'accept': 'application/json, text/plain, */*',
            'accept-language': 'zh-CN,zh;q=0.9',
            'cookie': 'ttwid=1%7CMzira2CT1P-CLey42gr0QsEGL_Wmq3Yg5PQF2X412hY%7C1677897397%7C0df7a1da2a44ccac7dda848d236c8d5276d3eae070dfb3fe6df6e86fbd896d93;',
            'priority': 'u=1, i',
            'referer': 'https://live.douyin.com/7254458840',
            'sec-ch-ua': '"Chromium";v="140", "Not=A?Brand";v="24", "Google Chrome";v="140"',
            'sec-ch-ua-mobile': '?0',
            'sec-ch-ua-platform': '"macOS"',
            'sec-fetch-dest': 'empty',
            'sec-fetch-mode': 'cors',
            'sec-fetch-site': 'same-origin',
            'user-agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36',
            // 'x-secsdk-csrf-token': '111100000001867a1807da8f81910d0469f2b185299bfe74a32e18'
        };
        
        // 发送请求
        const response = await axios.get(url, {
            headers: headers,
            timeout: 30000
        });
        
        // 生成带时间戳的文件名
        const now = new Date();
        const timestamp = now.getFullYear() + 
                         String(now.getMonth() + 1).padStart(2, '0') + 
                         String(now.getDate()).padStart(2, '0') + '_' +
                         String(now.getHours()).padStart(2, '0') + 
                         String(now.getMinutes()).padStart(2, '0') + 
                         String(now.getSeconds()).padStart(2, '0');
        const filename = `douyin_data_${timestamp}.json`;
        const filepath = path.join(__dirname, filename);
        
        // 将响应保存到JSON文件
        await fs.writeFile(filepath, JSON.stringify(response.data, null, 2));
        
        // 获取文件大小
        const stats = await fs.stat(filepath);
        const fileSizeInBytes = stats.size;
        
        console.log(`数据获取成功！结果已保存到: ${filename}`);
        console.log(`文件大小: ${fileSizeInBytes} 字节`);
        
        // 显示响应预览（对应shell脚本中的head -5）
        console.log('\n响应预览:');
        const responseStr = JSON.stringify(response.data, null, 2);
        const previewLines = responseStr.split('\n').slice(0, 5);
        console.log(previewLines.join('\n'));
        
        return {
            success: true,
            filename: filename,
            fileSize: fileSizeInBytes,
            data: response.data
        };
        
    } catch (error) {
        console.error('错误: 请求执行失败');
        console.error('错误详情:', error.message);
        
        return {
            success: false,
            error: error.message
        };
    }
}

// 如果直接运行此文件，则执行函数
if (require.main === module) {
    fetchDouyinData()
        .then(result => {
            if (!result.success) {
                process.exit(1);
            }
        })
        .catch(error => {
            console.error('未处理的错误:', error);
            process.exit(1);
        });
}

// 导出函数供其他模块使用
module.exports = { fetchDouyinData };