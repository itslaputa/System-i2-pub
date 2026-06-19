import type { ParentProps } from "solid-js";
import type { AppPage } from "../app/routes";
import Footer from "../components/Footer";
import HeaderNav from "../components/HeaderNav";

type MainLayoutProps = ParentProps<{
  activePage: AppPage;
  onNavigate: (page: AppPage) => void;
}>;

function MainLayout(props: MainLayoutProps) {
  return (
    <div class="app-shell">
      <HeaderNav activePage={props.activePage} onNavigate={props.onNavigate} />
      <main class="app-main">{props.children}</main>
      <Footer />
    </div>
  );
}

export default MainLayout;
