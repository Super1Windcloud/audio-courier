import { Converter } from "opencc-js";

const t2s = Converter({ from: "tw", to: "cn" });



export  function convertTraditionalChinese(content :string ){
 return  t2s(content);
}