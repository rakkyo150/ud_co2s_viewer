import { invoke } from "@tauri-apps/api/tauri";
import Chart, { ChartOptions } from 'chart.js/auto';
import 'bootstrap/dist/css/bootstrap.min.css';
import { appDataDir } from '@tauri-apps/api/path';
import { path } from "@tauri-apps/api";

let address_data_file = "address_data.txt";

let inputAddress: HTMLInputElement | null;
let appropriateAddress: HTMLSpanElement | null;
let addressForm: HTMLFormElement | null;
let nowPpm: HTMLSpanElement | null;
let chart: HTMLCanvasElement | null;
let chart2: any;
let address: string | null = "";

window.addEventListener("DOMContentLoaded", () => {
  inputAddress = document.querySelector("#input-address");
  appropriateAddress = document.querySelector("#appropriate-address");
  addressForm = document.querySelector("#address-form");
  addressForm?.addEventListener("submit", (e) => {
    e.preventDefault();
    set_address();
  });
  nowPpm = document.querySelector("#now-ppm");
  chart = document.querySelector("#chart");
  
  appDataDir().then((result)=>{
    path.join(result, address_data_file).then((result)=>{
      address_data_file = result;
      invoke("read_file", { path: address_data_file }).then((result) => {
        if (result != null) {
          address = result as string;
        }

        make_chart().then((result) => {
          change_display(result);
        }).catch((error) => {
          console.log(error);
        });
      
        setInterval(async () => {
          if (addressForm!.style.display=="block" && appropriateAddress!.style.display=="none") return;
          let result = await make_chart()
          change_display(result);
        }, 5000);

      }).catch((error) => {
        console.error(error);
      });
    }).catch((error)=>{
      console.error(error);
    });
  });
});

function change_display(result: boolean) {
  if (!result) {
    addressForm!.style.display = "block";
    appropriateAddress!.style.display = "none";
  }
  else {
    addressForm!.style.display = "none";
    appropriateAddress!.style.display = "block";
  };
}

async function set_address() {
  if (addressForm!.style.display=="block" && inputAddress!.value!=null) {
    address = inputAddress!.value;
    invoke("write_file", { path: address_data_file, contents: address }).then((result) => {
      console.log(result);
    }).catch((error) => {
      console.error(error);
    });
    let result = await make_chart()
    change_display(result);
  }
}

async function make_chart(): Promise<boolean>{
  if(!nowPpm) return false;
  nowPpm.textContent = await invoke("ppm", { address: address });
  if(!chart) return false;
  const ppm = Number(nowPpm.textContent);
  if(isNaN(ppm)) return false;
  let backgroundColor: string;
  if(ppm > 3500){
    backgroundColor = 'rgba(128, 0, 128, 1)';
  }
  else if(ppm > 2500){
    backgroundColor = 'rgba(255, 0, 0, 1)';
  }
  else if(ppm > 1500){
    backgroundColor = 'rgba(255, 165, 0, 1)';
  }
  else if(ppm > 1000){
    backgroundColor = 'rgba(0, 128, 0, 1)';
  }
  else{
    backgroundColor = 'rgba(0, 0, 255, 1)';
  }
  const percentage = Number(nowPpm.textContent) / 3500 * 100;
  if(isNaN(percentage)) return false;
  const donutOptions: ChartOptions = {
    borderColor: 'rgba(211, 211, 211, 1)',
    animation: false,
  }
  if(chart2 == null){
    chart2 = new Chart(chart, {
      type: 'doughnut',
      data: {
        datasets: [{
          data: [percentage, 100 - percentage],
          backgroundColor: [
            backgroundColor, //赤色
            'rgba(0, 0, 0, 0)',
          ],
        }]
      },
      options: donutOptions
    });
  }
  else{
    chart2.destroy();
    chart2 = new Chart(chart, {
      type: 'doughnut',
      data: {
        datasets: [{
          data: [percentage, 100 - percentage],
          backgroundColor: [
            backgroundColor, //赤色
            'rgba(0, 0, 0, 0)',
          ],
          animation: false,
        }]
      },
      options: donutOptions,
    });
  }

  nowPpm.textContent += " ppm";
  return true;
}