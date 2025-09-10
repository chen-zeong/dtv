const axios = require('axios');
const crypto = require('crypto');
const { sign } = require("./Signer.js");

// 从index.js复制getTtwid函数
async function getTtwid() {
  try {
    const url = 'https://ttwid.bytedance.com/ttwid/union/register/';
    const data = {
      "region": "cn",
      "aid": 1768,
      "needFid": false,
      "service": "www.ixigua.com",
      "migrate_info": { "ticket": "", "source": "node" },
      "cbUrlProtocol": "https",
      "union": true
    };
    const response = await axios.post(url, data, { headers: { 'Content-Type': 'application/json' } });
    const setCookie = response.headers['set-cookie'];
    const regex = /ttwid=([^;]+)/;
    const match = regex.exec(setCookie[0]);
    return match && match.length > 1 ? match[1] : '';
  } catch (error) {
    console.error('获取ttwid时出错:', error.message);
    return '';
  }
}

// 测试函数
async function testGetTtwid() {
  console.log('正在生成ttwid...');
  
  try {
    const ttwid = await getTtwid();
    
    if (ttwid) {
      console.log('\n✅ 成功生成ttwid:');
      console.log('ttwid:', ttwid);
      console.log('\n📊 ttwid信息:');
      console.log('- 长度:', ttwid.length);
      console.log('- 类型:', typeof ttwid);
      console.log('- 时间戳:', new Date().toISOString());
      
      // 保存到文件
      const fs = require('fs');
      const result = {
        ttwid: ttwid,
        timestamp: new Date().toISOString(),
        length: ttwid.length
      };
      
      fs.writeFileSync('ttwid_result.json', JSON.stringify(result, null, 2));
      console.log('\n💾 结果已保存到 ttwid_result.json');
      
    } else {
      console.log('❌ 生成ttwid失败');
    }
  } catch (error) {
    console.error('❌ 测试过程中出错:', error.message);
  }
}

// 运行测试
testGetTtwid();