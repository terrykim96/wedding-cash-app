import { mount } from "svelte";
import App from "./App.svelte";
// 글꼴(Pretendard, SIL OFL 1.1)은 빌드 시 앱 안에 포함된다. 실행 PC에 설치할 필요 없음
import "pretendard/dist/web/variable/pretendardvariable.css";
import "./app.css";

const target = document.getElementById("app");
if (!target) throw new Error("#app 요소가 없습니다");

const app = mount(App, { target });

export default app;
