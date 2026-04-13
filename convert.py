import base64

def convert_ps1_to_base64(file_path):
    try:
        with open(file_path, 'rb') as f:
            content = f.read()
            # 编码为 base64 字节
            b64_bytes = base64.b64encode(content)
            # 转换为字符串
            b64_string = b64_bytes.decode('utf-8')

            print("--- Base64 String Start ---")
            print(b64_string)
            print("--- Base64 String End ---")

            # 同时也保存到文件方便复制
            with open("base64_output.txt", "w") as out:
                out.write(b64_string)
            print("\n[!] 字符串已保存在 base64_output.txt 中，直接复制即可。")

    except FileNotFoundError:
        print("错误：找不到 install-stm32tool.ps1 文件。")

if __name__ == "__main__":
    # 请确保 ps1 文件在同一目录下
    convert_ps1_to_base64('install-stm32tool.ps1')