import { For } from "solid-js";
import { APP_NAV_ITEMS, type AppPage } from "../app/routes";

type HeaderNavProps = {
  activePage: AppPage;
  onNavigate: (page: AppPage) => void;
};

function HeaderNav(props: HeaderNavProps) {
  return (
    <header class="app-header">
      <nav class="header-nav" aria-label="Primary">
        <For each={APP_NAV_ITEMS}>
          {(item) => (
            <button
              type="button"
              class="nav-button"
              classList={{ "is-active": props.activePage === item.id }}
              onClick={() => props.onNavigate(item.id)}
            >
              {item.label}
            </button>
          )}
        </For>
      </nav>
    </header>
  );
}

export default HeaderNav;
