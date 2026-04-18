"""Скачивание текстов произведений с lib.ru для корпуса авторов."""

import os
import time
import logging
from pathlib import Path

import requests
from bs4 import BeautifulSoup, NavigableString, Tag

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
logger = logging.getLogger(__name__)

DATA_DIR = Path(__file__).resolve().parent.parent / "data" / "raw"

AUTHORS = {
    "dostoevsky": {
        "base": "http://az.lib.ru/d/dostoewskij_f_m",
        "sources": [
            {"file": "text_0060.shtml", "title": "Преступление и наказание"},
            {"file": "text_0070.shtml", "title": "Идиот"},
            {"file": "text_0080.shtml", "title": "Бесы"},
            {"file": "text_0100.shtml", "title": "Братья Карамазовы ч1"},
            {"file": "text_0110.shtml", "title": "Братья Карамазовы ч2"},
            {"file": "text_0120.shtml", "title": "Братья Карамазовы ч3"},
            {"file": "text_0130.shtml", "title": "Братья Карамазовы ч4"},
            {"file": "text_0290.shtml", "title": "Записки из подполья"},
            {"file": "text_0010.shtml", "title": "Бедные люди"},
            {"file": "text_0020.shtml", "title": "Униженные и оскорбленные"},
            {"file": "text_0230.shtml", "title": "Белые ночи"},
        ],
    },
    "chekhov": {
        "base": "http://az.lib.ru/c/chehow_a_p",
        "sources": [
            {"file": "text_0010.shtml", "title": "Рассказы 1880-1882"},
            {"file": "text_0020.shtml", "title": "Рассказы 1883-1884"},
            {"file": "text_0030.shtml", "title": "Рассказы 1884-1885"},
            {"file": "text_0040.shtml", "title": "Рассказы 1885-1886"},
            {"file": "text_0050.shtml", "title": "Рассказы 1886"},
            {"file": "text_0060.shtml", "title": "Рассказы 1887"},
            {"file": "text_0070.shtml", "title": "Рассказы 1888-1891"},
            {"file": "text_0080.shtml", "title": "Рассказы 1892-1894"},
            {"file": "text_0090.shtml", "title": "Рассказы 1894-1897"},
            {"file": "text_0100.shtml", "title": "Рассказы 1898-1903"},
            {"file": "text_0120.shtml", "title": "Чайка"},
            {"file": "text_0130.shtml", "title": "Дядя Ваня"},
            {"file": "text_0140.shtml", "title": "Три сестры"},
            {"file": "text_0150.shtml", "title": "Вишневый сад"},
        ],
    },
    "bulgakov": {
        "base": "http://az.lib.ru/b/bulgakow_m_a",
        "sources": [
            {"file": "text_1924_belaya_gvardia.shtml", "title": "Белая гвардия"},
            {"file": "text_1924_diavoliada.shtml", "title": "Дьяволиада"},
            {"file": "text_01_1926_polotentze_s_petuhom.shtml", "title": "Полотенце с петухом"},
            {"file": "text_02_1926_viyuga.shtml", "title": "Вьюга"},
            {"file": "text_03_1925_stalnoe_gorlo.shtml", "title": "Стальное горло"},
            {"file": "text_04_1925_tma_egipetskaya.shtml", "title": "Тьма египетская"},
            {"file": "text_05_1925_kreshenie_povorotom.shtml", "title": "Крещение поворотом"},
            {"file": "text_09_1927_morfiy.shtml", "title": "Морфий"},
            {"file": "text_1924_bagroviy_ostrov.shtml", "title": "Багровый остров"},
            {"file": "text_1923_samogonnoe_ozero.shtml", "title": "Самогонное озеро"},
            {"file": "text_1922_krasnaya_korona.shtml", "title": "Красная корона"},
            {"file": "text_1922_v_noch_na_tretie_chislo.shtml", "title": "В ночь на 3-е число"},
            {"file": "text_1923_stolitza_v_bloknote.shtml", "title": "Столица в блокноте"},
            {"file": "text_1925_bogema.shtml", "title": "Богема"},
            {"file": "text_1921_nedelya_prosveshenia.shtml", "title": "Неделя просвещения"},
            {"file": "text_1922_chasha_zhizni.shtml", "title": "Чаша жизни"},
            {"file": "text_1925_dni_turbinyh.shtml", "title": "Дни Турбиных"},
        ],
    },
}

HEADERS = {
    "User-Agent": "Mozilla/5.0 (compatible; GlyphCorpusBot/1.0; academic research)"
}
DELAY = 2  # секунд между запросами


def extract_text(html: str) -> str:
    """Извлечение текста произведения из HTML-страницы lib.ru.

    На lib.ru теги <dd> вложены друг в друга, поэтому нельзя брать get_text()
    от родительского — он соберёт весь текст дочерних. Берём только прямые
    текстовые узлы и inline-теги каждого <dd>.
    """
    soup = BeautifulSoup(html, "lxml")

    dd_tags = soup.find_all("dd")
    if dd_tags:
        paragraphs = []
        for dd in dd_tags:
            parts = []
            for child in dd.children:
                if isinstance(child, NavigableString):
                    s = str(child).strip()
                    if s:
                        parts.append(s)
                elif isinstance(child, Tag) and child.name in (
                    "b", "i", "em", "strong", "a", "span",
                ):
                    s = child.get_text().strip()
                    if s:
                        parts.append(s)
            text = " ".join(parts).strip()
            if len(text) > 10:
                paragraphs.append(text)
        return "\n\n".join(paragraphs)

    # fallback: <pre> теги
    pre_tags = soup.find_all("pre")
    if pre_tags:
        return "\n\n".join(pre.get_text() for pre in pre_tags)

    return ""


def download_text(url: str) -> str | None:
    """Скачивание и декодирование страницы с lib.ru."""
    try:
        resp = requests.get(url, headers=HEADERS, timeout=30)
        resp.raise_for_status()
        # lib.ru использует windows-1251
        resp.encoding = "windows-1251"
        return resp.text
    except requests.RequestException as e:
        logger.error("Ошибка загрузки %s: %s", url, e)
        return None


def main():
    total_stats = {}

    for author, config in AUTHORS.items():
        author_dir = DATA_DIR / author
        author_dir.mkdir(parents=True, exist_ok=True)
        author_chars = 0

        for source in config["sources"]:
            title = source["title"]
            safe_title = title.replace(" ", "_").replace("/", "_")
            out_path = author_dir / f"{safe_title}.txt"

            # идемпотентность: не перезагружаем существующие файлы
            if out_path.exists() and out_path.stat().st_size > 0:
                chars = out_path.stat().st_size
                author_chars += chars
                logger.info("Пропуск (уже есть): %s — %d символов", title, chars)
                continue

            url = f"{config['base']}/{source['file']}"
            logger.info("Загрузка: %s — %s", author, title)

            html = download_text(url)
            if html is None:
                continue

            text = extract_text(html)
            if not text or len(text) < 100:
                logger.warning("Мало текста для %s: %d символов", title, len(text))
                continue

            out_path.write_text(text, encoding="utf-8")
            author_chars += len(text)
            logger.info("Сохранено: %s — %d символов", title, len(text))

            time.sleep(DELAY)

        total_stats[author] = author_chars
        logger.info("Итого %s: %d символов", author, author_chars)

    logger.info("=== Общая статистика ===")
    for author, chars in total_stats.items():
        logger.info("  %s: %d символов (%.1f MB)", author, chars, chars / 1_000_000)


if __name__ == "__main__":
    main()
