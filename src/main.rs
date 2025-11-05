use std::process::Command;
use std::fs;
use std::thread;
use std::time::Duration;

pub struct Xdotool{
    pub id:String,
    pub wm:String
}

impl Xdotool {
    fn init() -> Xdotool{
        let get_id = "xdotool search --classname cloudmusic.exe".to_string();
        let out = Xdotool::bash(&get_id);
        
        let id:&str = match out.find('\n') {
                // 找到换行符
                Some(index) => &out[0..index], 
                // 没找到，返回整个字符串
                None => "None", 
            };
        Xdotool { id: (id.to_string()), wm: (Xdotool::getwm(&id)) }
    }
    fn get_wm(&self) -> String{
        Xdotool::getwm(&self.id)
    }
    fn getwm(id :&str) -> String{
        let get_wm = format!("xprop -id {id} WM_NAME");
        let out = Xdotool::bash(&get_wm);
        let after_equals = out.split('=').nth(1).unwrap_or(&out);
        // 2. 移除所有空格
        let trimmed = after_equals.trim();
        // 3. 移除首尾的双引号
        let wm = trimmed.trim_matches('"');
        // println!("{wm}");
        wm.to_string()
    }

    fn bash(cmd:&String) -> String{
        let output = Command::new("sh") // 也可以使用 "bash"
            .arg("-c") // -c 选项告诉 shell 执行后面的字符串作为一个命令
            .arg(cmd)
            .output()
            .expect("Failed to execute shell command");
        if output.status.success() {
            // let target_id = stdout
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            
            // println!("Bash complate out: {}", stdout);
            return stdout.to_string();
        } else {
            // let stderr = String::from_utf8_lossy(&output.stderr);
            // eprintln!("未找到网易云音乐\n{}", stderr);
            return "None".to_string();
        } 
    }
}

fn write_file(filename: &str, content: &str){
    // fs::write() 会创建一个新文件，或覆盖现有文件的内容。
    let _ = fs::write(filename, content);
}
fn main() {
   
    let sleep = Duration::from_secs(1);
    let mut count =0;
    loop {

        let mut xdotool = Xdotool::init();
        xdotool.wm = "".to_string();
        // let mut wm = xdotool.wm.clone();
        loop {
            if xdotool.id == "None"{
                count = count+1;
                if count >1{
                    break;
                }
                eprintln!("未找到网易云音乐");
                write_file("title.txt","");
                break;
            }
            
            let temp = xdotool.get_wm();
            if xdotool.wm != temp{
                xdotool.wm = temp;
                write_file("title.txt",&xdotool.wm);
                println!("{}",&xdotool.wm);
            }
            xdotool = Xdotool::init();
            thread::sleep(sleep);
        }
        thread::sleep(sleep);
    }
}