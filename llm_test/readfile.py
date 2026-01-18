import os
cwd = os.getcwd()
print(cwd)

from pathlib import Path

current_path = Path.cwd()
parent_dir = current_path.parent
file_path = parent_dir / 'rustscript' / 'RustScript手册.md'

print(file_path)

content = file_path.read_text(encoding='utf-8')
print(content)

import os
# 升级方舟 SDK 到最新版本 pip install -U 'volcengine-python-sdk[ark]'
from volcenginesdkarkruntime import Ark

client = Ark(
    # 从环境变量中读取您的方舟API Key
    api_key="906ae4a7-49f1-44f7-a2e4-5b9c0bc54948",
    # 深度思考模型耗费时间会较长，请您设置较大的超时时间，避免超时，推荐30分钟以上
    timeout=1800,
    )

response = client.chat.completions.create(
    # 替换 <Model> 为您的Model ID
    model="doubao-seed-1-6-251015",
    messages=[
        {"role": "user", "content": "根据RustScript手册，用RustScript的语法，实现电力系统潮流计算\n" + content}
    ],
     thinking={
         "type": "disabled" # 不使用深度思考能力,
         # "type": "enabled" # 使用深度思考能力
         # "type": "auto" # 模型自行判断是否使用深度思考能力
     },
)


print(response.choices[0].message.content)
