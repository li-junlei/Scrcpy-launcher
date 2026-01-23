use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use anyhow::{Result, bail, Context};

/// 定义进度回调的类型：已传输字节, 总字节
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send + Sync>;

pub struct AdbPusher {
    device_serial: Option<String>,
}

impl AdbPusher {
    pub fn new(serial: Option<String>) -> Self {
        Self { device_serial: serial }
    }

    /// 连接 ADB Server 并建立 Transport
    async fn connect(&self) -> Result<TcpStream> {
        let mut stream = TcpStream::connect("127.0.0.1:5037").await
            .context("无法连接到 ADB Server，请确保 adb start-server 已运行")?;

        // 1. 切换到指定设备 (host:transport:serial)
        let target = match &self.device_serial {
            Some(s) => format!("host:transport:{}", s),
            None => "host:transport-any".to_string(), // 默认连接第一个设备
        };
        self.send_packet(&mut stream, &target).await?;
        self.read_status(&mut stream).await.context("设备连接失败或未授权")?;

        Ok(stream)
    }

    /// 发送 ADB 格式的数据包 (4字节长度 + 内容)
    async fn send_packet(&self, stream: &mut TcpStream, payload: &str) -> Result<()> {
        let len_str = format!("{:04x}", payload.len());
        stream.write_all(len_str.as_bytes()).await?;
        stream.write_all(payload.as_bytes()).await?;
        Ok(())
    }

    /// 读取 ADB 的 OKAY/FAIL 状态
    async fn read_status(&self, stream: &mut TcpStream) -> Result<()> {
        let mut status = [0u8; 4];
        stream.read_exact(&mut status).await?;
        if &status == b"OKAY" {
            Ok(())
        } else {
            // 如果失败，读取错误信息
            let mut len_buf = [0u8; 4];
            if stream.read_exact(&mut len_buf).await.is_ok() {
                 let len_str = String::from_utf8_lossy(&len_buf);
                 if let Ok(len) = u64::from_str_radix(&len_str, 16) {
                     let mut err_msg = vec![0u8; len as usize];
                     let _ = stream.read_exact(&mut err_msg).await;
                     bail!("ADB Error: {}", String::from_utf8_lossy(&err_msg));
                 }
            }
            bail!("ADB 返回 FAIL")
        }
    }

    /// 核心功能：带进度的 Push
    pub async fn push(&self, local_path: &str, remote_path: &str, callback: Option<ProgressCallback>) -> Result<()> {
        let path = Path::new(local_path);
        let file_size = path.metadata()?.len();
        let mut file = File::open(path).await?;
        let mut buffer = [0u8; 64 * 1024]; // 64KB Chunk size (ADB 推荐)

        // 1. 连接并进入 SYNC 模式
        let mut stream = self.connect().await?;
        self.send_packet(&mut stream, "sync:").await?;
        self.read_status(&mut stream).await.context("无法进入 SYNC 模式")?;

        // 2. 发送 SEND 请求 (ID_SEND + 长度 + 远程路径)
        let remote_path_bytes = remote_path.as_bytes();
        stream.write_all(b"SEND").await?;
        stream.write_u32_le(remote_path_bytes.len() as u32).await?;
        stream.write_all(remote_path_bytes).await?;

        // 3. 循环发送 DATA 数据块
        let mut total_sent = 0u64;
        
        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 { break; } // 文件读完

            // 发送数据头: DATA + 块大小
            stream.write_all(b"DATA").await?;
            stream.write_u32_le(n as u32).await?;
            // 发送数据体
            stream.write_all(&buffer[..n]).await?;

            // === 触发进度回调 ===
            total_sent += n as u64;
            if let Some(cb) = &callback {
                cb(total_sent, file_size);
            }
        }

        // 4. 发送 DONE (结束 + 修改时间)
        let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();
        stream.write_all(b"DONE").await?;
        stream.write_u32_le(time as u32).await?;

        // 5. 等待服务器确认 OKAY
        let mut resp = [0u8; 4];
        stream.read_exact(&mut resp).await?;
        if &resp != b"OKAY" {
            bail!("传输未被确认，可能失败");
        }

        Ok(())
    }
}
