# [Решение японских кроссвордов с P̶y̶t̶h̶o̶n̶ Rust и WebAssembly](https://habr.com/ru/post/454586/)

![Rust logo as nonogram](https://habrastorage.org/webt/sy/k0/va/syk0va4uczwmji3lwzhq2nqq2hy.png)

Как сделать решатель (солвер) нонограмм на Python, переписать его на Rust, чтобы запускать прямо в браузере через WebAssembly.

[TL;DR](https://tsionyx.github.io/nono/?id=32480)

<cut />

## Начало

Про японские кроссворды (нонограммы) на хабре было уже несколько постов. [Пример](https://habr.com/ru/post/418069/)
и [еще один](https://habr.com/ru/post/433330/).

> Изображения зашифрованы числами, расположенными слева от строк, а также сверху над столбцами. Количество чисел показывает, сколько групп чёрных (либо своего цвета, для цветных кроссвордов) клеток находятся в соответствующих строке или столбце, а сами числа — сколько слитных клеток содержит каждая из этих групп (например, набор из трёх чисел — 4, 1, и 3 означает, что в этом ряду есть три группы: первая — из четырёх, вторая — из одной, третья — из трёх чёрных клеток). В чёрно-белом кроссворде группы должны быть разделены, как минимум, одной пустой клеткой, в цветном это правило касается только одноцветных групп, а разноцветные группы могут быть расположены вплотную (пустые клетки могут быть и по краям рядов). Необходимо определить размещение групп клеток.


Одна из наиболее общепринятых точек зрения - что "правильными" кроссвордами можно называть только те, которые решаются "логическим" путем. Обычно так называют способ решения, при котором не принимаются во внимание зависимости между разными строками и/или столбцами. Иначе говоря, решение представляет собой последовательность **независимых** решений отдельных строк или столбцов, пока все клетки не окажутся закрашены (подробнее об алгоритме далее). Например только такие нонограммы можно найти на сайте http://nonograms.org/ (http://nonograms.ru/). Нонограммы с этого сайта уже приводились в качестве примера в статье [Решение цветных японских кроссвордов со скоростью света](https://habr.com/ru/post/418069/). Для целей сравнения и проверки в моем солвере также добавлена поддержка скачивания и парсинга кроссвордов с этого сайта (спасибо @KyberPrizrak за разрешение использовать материалы с его сайта).

Однако можно расширить понятие нонограмм до более общей задачи, когда обычный "логический" способ заводит в тупик. В таких случаях для решения приходится делать предположение о цвете какой-нибудь клетки и после, доказав, что этот цвет приводит к противоречию, отмечать для этой клетки противоположный цвет. Последовательность таких шагов может (если хватит терпения) выдать нам все решения. О решении такого более общего случая кроссвордов и будет главным образом эта статья.


## Python

Около полутора лет назад я случайно наткнулся на [статью](http://window.edu.ru/resource/781/57781), в которой рассказывался метод решения одной строки (как оказалось позднее, метод был довольно медленным).

Когда я реализовал этот метод на Python (мой основной рабочий язык) и добавил последовательное обновление всех строк, то увидел, что решается все это не слишком быстро. После изучения матчасти обнаружилось, что по этой теме существует масса работ и реализаций, которые предлагают разные подходы для этой задачи.

Как мне показалось, наиболее масштабную работу по анализу различных реализаций солверов провел Jan Wolter, опубликовав на своем сайте (который, насколько мне известно, остается самым крупным публичным хранилищем нонограмм в интернете) [подробное исследование](https://webpbn.com/survey/), содержащее огромное количество информации и ссылок, которые могут помочь в создании своего солвера.

Изучая многочисленные источники (будут в конце статьи), я постепенно улучшал скорость и функциональность моего солвера. В итоге меня затянуло и я занимался реализацией, рефакторингом, отладкой алгоритмов около 10 месяцев в сводобное от работы время.

### Основные алгоритмы

Полученный солвер можно представить в виде четырех уровней решения:

- (**line**) линейный солвер: на входе строка из клеток и строка описания (clues), на выходе - частично решенная строка. В python-решении я реализовал 4 различных алгоритма (3 их них адаптированы для цветных кроссвордов). Самым быстрым оказался алгоритм BguSolver, названный в честь [первоисточника](https://www.cs.bgu.ac.il/~benr/nonograms/). Это очень эффективный и фактически стандартный метод решения нонограмм-строки при помощи динамического программирования. Псевдокод этого метода можно найти например [в этой статье](https://habr.com/ru/post/418069/#odna-stroka-dva-cveta).

- (**propagation**) все строки и столбцы складываем в очередь, проходим по ней линейным солвером, при получении новой информации при решении строки (столбца) обновляем очередь, соответственно, новыми столбцами (строками). Продолжаем, пока очередь не опустеет.

    <spoiler title="Пример и код">
    Берем очередную задачу для решения из очереди. Пусть это будет пустая (нерешенная) строка длины 7 (обозначим ее как <code>???????</code>) с описанием блоков <code>[2, 3]</code>. Линейный солвер выдаст частично решенную строку <code>?X??XX?</code>, где <code>X</code> - закрашенная клетка. При обновлении строки видим, что изменились столбцы с номерами 1, 4, 5 (индексация начинается с 0). Значит в указанных столбцах появилась новая информация и их можно заново отдавать "линейному" солверу. Складываем эти столбцы в очередь задач с повышенным приоритетом (чтобы отдать их линейному солверу следующими).
    <source lang="python">
    def propagation(board):
        line_jobs = PriorityDict()

        for row_index in range(board.height):
            new_job = (False, row_index)
            line_jobs[new_job] = 0

        for column_index in range(board.width):
            new_job = (True, column_index)
            line_jobs[new_job] = 0

        for (is_column, index), priority in line_jobs.sorted_iter():
            new_jobs = solve_and_update(board, index, is_column)

            for new_job in new_jobs:
                # upgrade priority
                new_priority = priority - 1
                line_jobs[new_job] = new_priority

    def solve_and_update(board, index, is_column):
        if is_column:
            row_desc = board.columns_descriptions[index]
            row = tuple(board.get_column(index))
        else:
            row_desc = board.rows_descriptions[index]
            row = tuple(board.get_row(index))

        updated = line_solver(row_desc, row)

        if row != updated:
            for i, (pre, post) in enumerate(zip(row, updated)):
                if _is_pixel_updated(pre, post):
                    yield (not is_column, i)

            if is_column:
                board.set_column(index, updated)
            else:
                board.set_row(index, updated)
    </source>
    </spoiler>

- (**probing**) для каждой нерешенной клетки перебираем все варианты цветов и пробуем propagation с этой новой информацией. Если получаем противоречие - выкидываем этот цвет из вариантов цветов для клетки и пытаемся извлечь из этого пользу снова при помощи propagation. Если решается до конца - добавляем решение в список решений, но продолжаем эксперименты с другими цветами (решений может быть несколько). Если приходим к ситуации, где дальше решить невозможно - просто игнорируем и повторяем процедуру с другим цветом/клеткой.
    <spoiler title="Код">
    Возвращает True, если в результате пробы было получено противоречие.
    <source lang="python">
    def probe(self, cell_state):
        board = self.board

        pos, assumption = cell_state.position, cell_state.color
        # already solved
        if board.is_cell_solved(pos):
            return False

        if assumption not in board.cell_colors(pos):
            LOG.warning("The probe is useless: color '%s' already unset", assumption)
            return False

        save = board.make_snapshot()

        try:
            board.set_color(cell_state)
            propagation(
                board,
                row_indexes=(cell_state.row_index,),
                column_indexes=(cell_state.column_index,))
        except NonogramError:
            LOG.debug('Contradiction', exc_info=True)
            # rollback solved cells
            board.restore(save)

        else:
            if board.is_solved_full:
                self._add_solution()

            board.restore(save)
            return False

        LOG.info('Found contradiction at (%i, %i)', *pos)
        try:
            board.unset_color(cell_state)
        except ValueError as ex:
            raise NonogramError(str(ex))

        propagation(
            board,
            row_indexes=(pos.row_index,),
            column_indexes=(pos.column_index,))

        return True
    </source>
    </spoiler>

- (**backtracking**) если при пробинге не игнорировать частично решенный пазл, а продолжать рекурсивно вызывать эту же процедуру - получим бэктрэкинг (иначе говоря - полный обход в глубину дерева потенциальных решений). Здесь начинает играть большую роль, какая из клеток будет выбрана в качестве следующего расширения потенциального решения. Хорошее исследование на эту тему есть [в этой публикации](https://ir.nctu.edu.tw/bitstream/11536/22772/1/000324586300005.pdf).

    <spoiler title="Код">
    Бэктрэкинг у меня довольно неряшливый, но вот эти две функции приблизительно описывают, что происходит при рекурсивном поиске
    <source lang="python">
    def search(self, search_directions, path=()):
        """
        Return False if the given path is a dead end (no solutions can be found)
        """
        board = self.board
        depth = len(path)

        save = board.make_snapshot()
        try:
            while search_directions:
                state = search_directions.popleft()

                assumption, pos = state.color, state.position
                cell_colors = board.cell_colors(pos)

                if assumption not in cell_colors:
                    LOG.warning("The assumption '%s' is already expired. "
                                "Possible colors for %s are %s",
                                assumption, pos, cell_colors)
                    continue

                if len(cell_colors) == 1:
                    LOG.warning('Only one color for cell %r left: %s. Solve it unconditionally',
                                pos, assumption)

                    try:
                        self._solve_without_search()
                    except NonogramError:
                        LOG.warning(
                            "The last possible color '%s' for the cell '%s' "
                            "lead to the contradiction. "
                            "The path %s is invalid", assumption, pos, path)
                        return False

                    if board.is_solved_full:
                        self._add_solution()
                        LOG.warning(
                            "The only color '%s' for the cell '%s' lead to full solution. "
                            "No need to traverse the path %s anymore", assumption, pos, path)
                        return True
                    continue

                rate = board.solution_rate
                guess_save = board.make_snapshot()
                try:
                    LOG.warning('Trying state: %s (depth=%d, rate=%.4f, previous=%s)',
                                state, depth, rate, path)
                    success = self._try_state(state, path)
                finally:
                    board.restore(guess_save)

                if not success:
                    try:
                        LOG.warning(
                            "Unset the color %s for cell '%s'. Solve it unconditionally",
                            assumption, pos)
                        board.unset_color(state)
                        self._solve_without_search()
                    except ValueError:
                        LOG.warning(
                            "The last possible color '%s' for the cell '%s' "
                            "lead to the contradiction. "
                            "The whole branch (depth=%d) is invalid. ", assumption, pos, depth)
                        return False

                    if board.is_solved_full:
                        self._add_solution()
                        LOG.warning(
                            "The negation of color '%s' for the cell '%s' lead to full solution. "
                            "No need to traverse the path %s anymore", assumption, pos, path)
                        return True
        finally:
            # do not restore the solved cells on a root path - they are really solved!
            if path:
                board.restore(save)

        return True

    def _try_state(self, state, path):
        board = self.board
        full_path = path + (state,)

        probe_jobs = self._get_all_unsolved_jobs(board)
        try:
            # update with more prioritized cells
            for new_job, priority in self._set_guess(state):
                probe_jobs[new_job] = priority

            __, best_candidates = self._solve_jobs(probe_jobs)
        except NonogramError as ex:
            LOG.warning('Dead end found (%s): %s', full_path[-1], str(ex))
            return False

        rate = board.solution_rate
        LOG.info('Reached rate %.4f on %s path', rate, full_path)

        if rate == 1:
            return True

        cells_left = round((1 - rate) * board.width * board.height)
        LOG.info('Unsolved cells left: %d', cells_left)

        if best_candidates:
            return self.search(best_candidates, path=full_path)

        return True
    </source>
    </spoiler>


Итак, мы начинаем решать наш кроссворд со второго уровня (первый годится только для вырожденного случая, когда во всем кроссворде только одна строка или столбец) и постепенно продвигаемся вверх по уровням. Как можно догадаться, каждый уровень несколько раз вызывает нижележащий уровень, поэтому для эффективного решения крайне необходимо иметь быстрые первый и второй уровень, которые для сложных пазлов могут вызываться миллионы раз.

На этом этапе выясняется (довольно ожидаемо), что python - это совсем не тот инструмент, который подходит для максимальной производительности в такой CPU-intensive задаче: все расчеты в нем крайне неэффективны по сравнению с более низкоуровневыми языками. Например, наиболее алгоритмически близкий BGU-солвер (на Java) по результатам замеров оказался в 7-17 (иногда до 27) раз быстрее на самых разных задачах.

<spoiler title="Подробнее">

<pre>
        pynogram_my BGU_my      speedup
Dancer      0.976    0.141      6.921986
Cat         1.064    0.110      9.672727
Skid        1.084    0.101     10.732673
Bucks       1.116    0.118      9.457627
Edge        1.208    0.094     12.851064
Smoke       1.464    0.120     12.200000
Knot        1.332    0.140      9.514286
Swing       1.784    0.138     12.927536
Mum         2.108    0.147     14.340136
DiCap       2.076    0.176     11.795455
Tragic      2.368    0.265      8.935849
Merka       2.084    0.196     10.632653
Petro       2.948    0.219     13.461187
M&M         3.588    0.375      9.568000
Signed      4.068    0.242     16.809917
Light       3.848    0.488      7.885246
Forever   111.000   13.570      8.179808
Center      5.700    0.327     17.431193
Hot         3.150    0.278     11.330935
Karate      2.500    0.219     11.415525
9-Dom     510.000   70.416      7.242672
Flag      149.000    5.628     26.474769
Lion       71.000    2.895     24.525043
Marley     12.108    4.405      2.748695
Thing     321.000   46.166      6.953169
Nature      inf    433.138      inf
Sierp       inf      inf        NaN
Gettys      inf      inf        NaN
</pre>

Замеры проводились на моей машине, пазлы взяты из стандартного набора, который использовал Jan Wolter в своем [сравнении](https://webpbn.com/survey/#samptime)
</spoiler>

И это уже после того, как я начал использовать PyPy, а на стандартном CPython время расчетов было выше, чем на PyPy в 4-5 раз! Можно сказать, что производительность похожего солвера на Java оказалась выше кода на CPython в 28-85 раз.

Попытки улучшить производительность моего солвера при помощи профайлинга (cProfile, SnakeViz, line_profiler) привели к некоторому ускорению, но сверхъестественного результата конечно не дали.

### [Итоги](https://github.com/tsionyx/pynogram):

**+** солвер умеет решать все пазлы с сайтов https://webpbn.com, http://nonograms.org и свой собственный (ini-based) формат

**+** решает черно-белые и цветные нонограммы с любым количеством цветов (максимальное количество цветов, которое встречалось - 10)

**+** решает пазлы с пропущенными размерами блоков (blotted). [Пример такого пазла](https://webpbn.com/19407).

**+** умеет рендерить пазлы в консоль / в окно curses / в браузер (при установке дополнительной опции _pynogram-web_). Для всех режимов поддерживается просмотр прогресса решения в реальном времени.

**-** медленные расчеты (в сравнении с реализациями, описанными в статье-сравнении солверов, см. [таблицу](https://webpbn.com/survey/#samptime)).

**-** неэффективный бэктрэкинг: некоторые пазлы могут решаться часами (когда дерево решений очень большое).


## Rust

В начале года я начал изучать Rust. Начал я, как водится, с [The Book](https://doc.rust-lang.org/book/), узнал про WASM, прошел [предлагаемый туториал](https://rustwasm.github.io/docs/book/). Однако хотелось какой-то настоящей задачи, в которой можно зайдействовать сильные стороны языка (в первую очередь его супер-производительность), а не выдуманных кем-то примеров. Так я снова вернулся к нонограммам. Но теперь у меня уже был работающий вариант всех алгоритмов на Python, его осталось "всего лишь" переписать.

С самого начала меня ожидала приятная новость: оказалось что Rust с его системой типов отлично описывает структуры данных для моей задачи. Так например одно из базовых соответствий _BinaryColor + BinaryBlock_ / _MultiColor + ColoredBlock_ позволяет навсегда разделить черно-белые и цветные нонограммы. Если где-то в коде мы попытаемся решить цветную строку при помощи обычных бинарных блоков описания, то получим ошибку компиляции про несоответствие типов.

<spoiler title="Базовые типы выглядят примерно так">
<source lang="rust">
pub trait Color
{
    fn blank() -> Self;
    fn is_solved(&self) -> bool;
    fn solution_rate(&self) -> f64;

    fn is_updated_with(&self, new: &Self) -> Result<bool, String>;

    fn variants(&self) -> Vec<Self>;
    fn as_color_id(&self) -> Option<ColorId>;
    fn from_color_ids(ids: &[ColorId]) -> Self;
}

pub trait Block
{
    type Color: Color;

    fn from_str_and_color(s: &str, color: Option<ColorId>) -> Self {
        let size = s.parse::<usize>().expect("Non-integer block size given");
        Self::from_size_and_color(size, color)
    }

    fn from_size_and_color(size: usize, color: Option<ColorId>) -> Self;

    fn size(&self) -> usize;
    fn color(&self) -> Self::Color;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Description<T: Block>
where
    T: Block,
{
    pub vec: Vec<T>,
}

// for black-and-white puzzles

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd)]
pub enum BinaryColor {
    Undefined,
    White,
    Black,
    BlackOrWhite,
}

impl Color for BinaryColor {
    // omitted
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone)]
pub struct BinaryBlock(pub usize);

impl Block for BinaryBlock {
    type Color = BinaryColor;
    // omitted
}

// for multicolor puzzles

#[derive(Debug, PartialEq, Eq, Hash, Default, Copy, Clone, PartialOrd, Ord)]
pub struct MultiColor(pub ColorId);

impl Color for MultiColor {
    // omitted
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone)]
pub struct ColoredBlock {
    size: usize,
    color: ColorId,
}

impl Block for ColoredBlock {
    type Color = MultiColor;
    // omitted
}
</source>
</spoiler>



При переносе кода некоторые моменты явно указывали на то, что статически типизированный язык, такой как Rust (ну или, например C++) - более подходящий для этой задачи. Точнее говоря, дженерики и трэйты лучше описывают предметную область чем иерархии классов. Так в Python-коде у меня было два класса для линейного солвера, <code>BguSolver</code> и <code>BguColoredSolver</code> которые решали, соответственно, черно-белую строку и цветную строку. В Rust-коде у меня осталась единственная generic-структура <code>struct DynamicSolver<B: Block, S = <B as Block>::Color></code>, которая умеет решать оба типа задач, в зависимости от переданного при создании типа (<code>DynamicSolver<BinaryBlock>, DynamicSolver<ColoredBlock></code>). Это, конечно, не значит, что в Python что-то похожее невозможно сделать, просто в Rust система типов явно указала мне, что если не пойти эти путем, то придется написать тонну повторяющегося кода.

К тому же любой кто пробовал писать на Rust, несомненно заметил эффект "доверия" к компилятору, когда процесс написания кода сводится к следующему псевдометаалгоритму:

<pre>
write_initial_code
while (compiler_hints = $(cargo check)) != 0; do
    fix_errors(compiler_hints)
end
</pre>

Когда компилятор перестанет выдавать ошибки и предупреждения, ваш код будет согласован с системой типов и borrow checker-ом и вы заранее предупредите возникновение кучи потенциальных багов (конечно, при условии тщательного проектирования типов данных).

Приведу пару примеров функций, которые показывают насколько лаконичен может быть код на Rust (в сравнении с Python-аналогами).

<spoiler title="unsolved_neighbours">
Выдает список нерешенных "соседей" для данной точки (x, y)

<source lang="python">
def unsolved_neighbours(self, position):
    for pos in self.neighbours(position):
        if not self.is_cell_solved(*pos):
            yield pos
</source>

<source lang="rust">
fn unsolved_neighbours(&self, point: &Point) -> impl Iterator<Item = Point> + '_ {
    self.neighbours(&point)
        .into_iter()
        .filter(move |n| !self.cell(n).is_solved())
}
</source>
</spoiler>

<spoiler title="partial_sums">
Для набора блоков, описывающих одну строку, выдать частичные суммы (с учетом обязательных промежутков между блоками).Полученные индексы будут указывать минимальную позицию, на которой блок может закончиться (эта информация используется далее для линейного солвера).

Например для такого набора блоков <code>[2, 3, 1]</code> имеем на выходе <code>[2, 6, 8]</code>, что означает, что первый блок может быть максимально сдвинут влево настолько, чтобы его правый край занимал 2-ую клетку, аналогично и для остальных блоков:

<pre>
            1 2 3 4 5 6 7 8 9
            _ _ _ _ _ _ _ _ _
     2 3 1 |_|_|_|_|_|_|_|_|_|
              ^       ^   ^
              |       |   |
конец 1 блока |       |   |
конец 2 блока --------    |
конец 3 блока ------------
</pre>

<source lang="python">
@expand_generator
def partial_sums(blocks):
    if not blocks:
        return

    sum_so_far = blocks[0]
    yield sum_so_far

    for block in blocks[1:]:
        sum_so_far += block + 1
        yield sum_so_far
</source>

<source lang="rust">
fn partial_sums(desc: &[Self]) -> Vec<usize> {
    desc.iter()
        .scan(None, |prev, block| {
            let current = if let Some(ref prev_size) = prev {
                prev_size + block.0 + 1
            } else {
                block.0
            };
            *prev = Some(current);
            *prev
        })
        .collect()
}
</source>

</spoiler>


При портировании я допустил несколько изменений

- ядро солвера (алгоритмы) подверглись незначительным изменениям (в первую очередь для поддержки generic-типов для клеток и блоков)
- оставил единственный (самый быстрый) алгоритм для линейного солвера
- вместо ini формата ввел чуть измененный TOML-формат
- не добавил поддержку blotted-кроссвордов, потому что, строго говоря, это уже другой класс задач
- оставил единственный способ вывода - просто в консоль, но теперь цветные клетки в консоли рисуются действительно цветными (благодаря [этому крэйту](https://crates.io/crates/colored))

  <spoiler title="вот так">
  ![Jack Sparrow](https://habrastorage.org/webt/xm/fd/ez/xmfdezlfahkoksuj3h0djom3p9k.png)
  </spoiler>


### Полезные инструменты

- [clippy](https://github.com/rust-lang/rust-clippy) - стандартный статический анализатор, который иногда даже может дать советы, слегка увеличивающие производительность кода
- [valgrind](http://www.valgrind.org/) - инструмент для динамического анализа приложений. Я использовал его как профайлер для поиска боттлнеков (<code>valrgind --tool=callgrind</code>) и особо прожорливых по памяти участков кода (<code>valrgind --tool=massif</code>). Совет: устанавливайте _[profile.release] debug=true_ в Cargo.toml перед запуском профайлера. Это позволит оставить debug-символы в исполняемом файле.
- [kcachegrind](https://github.com/KDE/kcachegrind) для просмотра файлов callgrind. Очень полезный инструмент для поиска наиболее проблемных с точки зрения производительности мест.


### Производительность

То, ради чего и затевалось переписывание на Rust. Берем кросвворды из уже упомянутой [таблицы сравнения](https://webpbn.com/survey/#samptime) и прогоняем их через лучшие, описанные в оригинальной статье, солверы. Результаты и описание прогонов [здесь](https://github.com/tsionyx/nonogrid/tree/dev/benches). Берем полученный [файл](https://github.com/tsionyx/nonogrid/blob/dev/benches/perf.csv) и строим на нем пару графиков.Так как время решения варьируется от миллисекунд до десятков минут, график выполнен с логарифмической шкалой.

<spoiler title="запускать в Jupyter-ноутбуке">

<source lang="python">
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
%matplotlib inline

# strip the spaces
df = pd.read_csv('perf.csv', skipinitialspace=True)
df.columns = df.columns.str.strip()
df['name'] = df['name'].str.strip()

# convert to numeric
df = df.replace('\+\ *', np.inf, regex=True)
ALL_SOLVERS = list(df.columns[3:])
df.loc[:,ALL_SOLVERS] = df.loc[:,ALL_SOLVERS].apply(pd.to_numeric)
# it cannot be a total zero
df = df.replace(0, 0.001)
#df.set_index('name', inplace=True)

SURVEY_SOLVERS = [s for s in ALL_SOLVERS if not s.endswith('_my')]
MY_MACHINE_SOLVERS = [s for s in ALL_SOLVERS if s.endswith('_my') and s[:-3] in SURVEY_SOLVERS]
MY_SOLVERS = [s for s in ALL_SOLVERS if s.endswith('_my') and s[:-3] not in SURVEY_SOLVERS]

bar_width = 0.17
df_compare = df.replace(np.inf, 10000, regex=True)
plt.rcParams.update({'font.size': 20})


def compare(first, others):
    bars = [first] + list(others)
    index = np.arange(len(df))
    fig, ax = plt.subplots(figsize=(30,10))

    df_compare.sort_values(first, inplace=True)

    for i, column in enumerate(bars):
        ax.bar(index + bar_width*i, df_compare[column], bar_width, label=column[:-3])

    ax.set_xlabel("puzzles")
    ax.set_ylabel("Time, s (log)")
    ax.set_title("Compare '{}' with others (lower is better)".format(first[:-3]))
    ax.set_xticks(index + bar_width / 2)
    ax.set_xticklabels("#" + df_compare['ID'].astype(str) + ": " + df_compare['name'].astype(str))
    ax.legend()

    plt.yscale('log')
    plt.xticks(rotation=90)
    plt.show()
    fig.savefig(first[:-3] + '.png', bbox_inches='tight')


for my in MY_SOLVERS:
    compare(my, MY_MACHINE_SOLVERS)

compare(MY_SOLVERS[0], MY_SOLVERS[1:])
</source>
</spoiler>

##### python-солвер

<a href="https://habrastorage.org/webt/si/qb/5o/siqb5ohxk_bjjaulmkohy1uhzkw.png">
![pynogram-performance](https://habrastorage.org/webt/si/qb/5o/siqb5ohxk_bjjaulmkohy1uhzkw.png)
</a>
(_картинка кликабельна_)

Видим, что _pynogram_ здесь медленнее всех представленных солверов. Единственное исключение из этого правила - солвер [Tamura/Copris](http://bach.istc.kobe-u.ac.jp/copris/puzzles/nonogram/), основанный на SAT, который самые простые пазлы (левая часть графика) решает дольше, чем наш. Однако такова уж особенность SAT-солверов: они предназначены для супер сложных кроссвордов, в которых обычный солвер надолго застревает в бэктрэкинге. Это отчетливо видно на правой части графика, где _Tamura/Copris_ решает самые сложные пазлы в десятки и сотни раз быстрее всех остальных.

##### rust-солвер

<a href="https://habrastorage.org/webt/bh/be/cb/bhbecb0ccinpwfhysauyjkrcx24.png">
![nonogrid-performance](https://habrastorage.org/webt/bh/be/cb/bhbecb0ccinpwfhysauyjkrcx24.png)
</a>
(_картинка кликабельна_)

На этом графике видно, что _nonogrid_ на простых задачах справляется также или чуть хуже, чем высокопроизводительные солверы, написанные на C и С++ (_Wolter_ и _Syromolotov_). С усложнением задач, наш солвер примерно повторяет траекторию _BGU_-солвера (Java), но почти всегда опережает его примерно на порядок. На самых сложных задачах _Tamura/Copris_ как всегда впереди всех.


##### rust vs python

<a href="https://habrastorage.org/webt/ih/0b/2r/ih0b2rnmyk5o_rpgckz_5hkkifc.png">
![py-vs-rust-performance](https://habrastorage.org/webt/ih/0b/2r/ih0b2rnmyk5o_rpgckz_5hkkifc.png)
</a>
(_картинка кликабельна_)

Ну и наконец сравнение двух наших солверов, описанных здесь. Видно, что Rust-солвер почти всегда опережает на 1-3 порядка питоновский солвер.


### [Итоги](https://github.com/tsionyx/nonogrid):

**+** солвер умеет решать все пазлы с сайтов https://webpbn.com (кроме blotted - c частично скрытыми размерами блоков), http://nonograms.org и свой собственный (TOML-based) формат

**+** решает черно-белые и цветные нонограммы с любым количеством цветов

**+** умеет рендерить пазлы в консоль (цветные c webpbn.com рисует по настоящему цветными)

**+** работает быстро (в сравнении с реализациями, описанными в статье-сравнении солверов, см. таблицу).

**-** бэктрэкинг остался неэффективным, как и в Python-решении: некоторые пазлы (например [вот такой безобидный 20x20](https://webpbn.com/3620)) могут решаться часами (когда дерево решений очень большое). Возможно вместо бэктрэкинга стоит воспользоваться уже упоминавшимися на хабре [SAT-солверами](https://habr.com/ru/post/433330/). Правда единственный найденный мною [SAT-солвер на Rust](https://github.com/kmcallister/sat) на первый взгляд кажется недописанным и заброшенным.


## WebAssembly

Итак, переписывание кода на Rust дало свои плоды: солвер стал намного быстрее. Однако Rust нам предлагает еще одну невероятно крутую фичу: компиляцию в WebAssembly и возможность запускать свой код прямо в браузере.

Для реализации этой возможности существует специальный инструмент для Rust, который обеспечивает необходимые биндинги и генерирует за вас boilerplate для безболезненного запуска Rust функций в JS-коде - это _wasm-pack_ (+_wasm-bindgen_). Большая часть работы c ним и другими важными инструментами уже описана в [туториале Rust and WebAssembly](https://rustwasm.github.io/docs/book/). Однако есть пара моментов, которые пришлось выяснять самостоятельно:

- при чтении создается ощущение, что туториал в первую очередь написан для JS-девелопера, который хочет ускорить свой код при помощи Rust. Ну или по крайней мере, для того кто знаком с _npm_. Для меня же, как человека, далекого от фронтэнда, было удивлением обнаружить, что даже стандартный пример из книги никак не хочет работать со сторонним web-сервером, отличающимся от <code>npm run start</code>.

  К счастью в wasm-pack есть режим, позволяющий генерировать обычный JS-код (не являющийся npm-модулем). <code>wasm-pack build --target no-modules --no-typescript</code> на выходе даст всего два файла: _project-name.wasm_ - бинарник Rust-кода, скомпилированного в WebAssembly и _project-name.js_. Последний файл можно добавить на любую HTML-страницу <code><script src="project-name.js"></script></code> и использовать WASM-функции, не заморачиваясь с npm, webpack, ES6, модулями и прочими радостями современного JS-разработчика. Режим <code>no-modules</code> идеально подходит для не-фронтэндеров в процессе разработки WASM-приложения, а также для примеров и демонстраций, потому что не требует никакой дополнительной frontend-инфраструктуры.

- WebAssembly хорош для задач, которые слишком тяжелы для JavaScript. В первую очередь это задачи, которые выполняют множество расчетов. А раз так, такая задача может выполняться долго даже с WebAssembly и тем самым нарушить асинхронный принцип работы современного веба. Я говорю про всевозможные _Warning: Unresponsive script_, которые мне случилось наблюдать при работе моего солвера. Для решения этой проблемы можно использовать механизм _Web worker_. В таком случае схема работы с "тяжелыми" WASM-функциями может выглядеть так:

  1. из основного скрипта по событию (например клику на кнопке) послать сообщение воркеру с заданием запустить тяжелую функцию.
  2. воркер принимает задание, запускает функцию и по ее окончанию возвращает результат.
  3. основной скрипт принимает результат и как-то его обрабатывает (отрисовывает)

При создании WASM-интерфейса нет возможности передавать все создаваемые типы данных в JS, к тому же это противоречит [практике использования WASM](https://rustwasm.github.io/docs/book/game-of-life/implementing.html#interfacing-rust-and-javascript). Однако между вызовами функций нужно как-то хранить состояние (внутреннее представление нонограмм с описанием клеток и блоков), поэтому я использовал глобальный <code>HashMap</code> для хранения нонограмм по их порядковым номерам, который никак не виден снаружи. При запросе извне (из JS) передается только номер кроссворда, по которому затем восстанавливается сам кроссворд для запуска решения / запроса результатов решения.

Для обеспечения безопасного доступа к глобальному словарю, он [заворачивается в Mutex](https://github.com/tsionyx/nono/blob/8e2f8f27cce70492b66a929287e011b1ce357324/src/lib.rs#L45), что заставляет заменить все используемые структуры на thread-safe. Изменения в таком случае касаются использования smart-указателей в основном коде солвера. Для поддержки thread-safe операций пришлось заменить все _Rc_ на _Arc_ и _RefCell_ на _RwLock_. Однако эта операция тут же сказалась на производительности солвера: время выполнения по самой оптимистичной оценке увеличилось на 30%. Для обхода этого ограничения я добавил специальную опцию <code>--features=threaded</code> при необходимости использовать солвер в thread-safe среде, которая и необходима в WASM-интерфейсе.

В результате проведенных замеров на кроссвордах [6574](https://tsionyx.github.io/nono/?id=6574) и [8098](https://tsionyx.github.io/nono/?id=8098) получился следующий результат (лучшее время в секундах из 10 запусков):

| id   | non-thread-safe | thread-safe | web-interface |
|------|----------------|-------------|---------------|
| 6574 | 5.4            | 7.4         | 9.2           |
| 8098 | 21.5           | 28.4        | 29.9          |

Видно, что в веб-интерфейсе пазл решается на 40..70% медленнее, чем при запуске нативного приложения в консоли, причем большую часть этого замедления (32..37%) берет на себя запуск в thread-safe режиме (<code>cargo build --release --features=threaded</code>).

Тесты проводились в Firefox 67.0 и Chromium 74.0.

WASM-солвер можно попробовать [здесь](https://tsionyx.github.io/nono/) ([исходники](https://github.com/tsionyx/nono)). Интерфейс позволяет выбрать кроссворд по его номеру с одного из сайтов https://webpbn.com/ или http://www.nonograms.org/



### TODO

  - "выкидывание" решенных строк/столбцов, чтобы облегчить/ускорить решение на этапе бэктрэкинга.

  - если солвер находит несколько решений, то интерфейс их не показывает. Вместо этого он показывает максимально "общее" решение, то есть неполное решение, в котором незакрашенные клетки могут иметь разные значения. Нужно добавить показ всех найденных решений.

  - нет ограничения по времени (некоторые пазлы считаются очень долго, традиционно я запускал с таймаутом 3600 секунд). В WASM невозможно использовать системный вызов времени, чтобы запустить ограничивающий таймер (на самом деле, это единственное (!) [изменение](https://github.com/tsionyx/nonogrid/commit/47b48109927e3146455636df2c32efc44232733b), которое пришлось сделать, чтобы солвер заработал в WASM). Это, я уверен, как-то можно пофиксить, но возможно придется в основной крэйт nonogrid вносить изменения.

  - невозможно отслеживать прогресс. Здесь у меня есть некоторые наработки: коллбэки, которые могут срабатывать при изменении состояния клеток, но как их пробросить в WASM пока не думал. Возможно стоит создать очередь, привязанную к пазлу и писать в нее все (или основные) изменения на этапе решения, а со стороны JS сделать цикл, вычитывающий эту очередь и рендерящий изменения.

  - уведомления об ошибках в JS. Например при запросе несуществующего пазла в консоль вываливается backtrace, но на странице просто ничего не происходит.

  - добавить поддержку других источников и внешних нонограмм (например возможность загружать файлы в [TOML-формате](https://github.com/tsionyx/nonogrid/blob/master/examples/hello.toml))



## Итоги

- задача решения нонограмм позволила мне получить массу новых знаний об алгоритмах, языках и инфраструктуре (профайлерах, анализаторах, etc).

- солвер на Rust на 1-3 порядка быстрее солвера на PyPy при увеличении количества кода всего в 1.5-2 раза (точно не замерял).

- переносить код с Python на Rust достаточно просто, если он разбит на достаточно мелкие функции и использует функциональные возможности Python (итераторы, генераторы, comprehensions), которые замечательно транслируются в идиоматичный Rust-код.

- писать на Rust под WebAssembly можно уже сейчас. При этом производительность исполнения Rust кода, скомпилированного в WASM, довольно близка к нативному приложению (на моей задаче примерно в 1.5 раза медленнее).


## Основные источники

1. [The 'pbnsolve' Paint-by-Number Puzzle Solver by Jan Wolter](http://webpbn.com/pbnsolve.html) and the [survey](http://webpbn.com/survey/)

2. [The BGU Nonograms Project](https://www.cs.bgu.ac.il/~benr/nonograms/)

3. [Solving Nonograms by combining relaxations](http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.177.76&rep=rep1&type=pdf)

4. [An Efficient Approach to Solving Nonograms](https://ir.nctu.edu.tw/bitstream/11536/22772/1/000324586300005.pdf)

5. [Решение цветных японских кроссвордов со скоростью света](https://habr.com/post/418069)

6. [Color and black and white Japanese crosswords on-line](http://www.nonograms.org/)

7. [Решение японских кроссвордов с использованием конечных автоматов](http://window.edu.ru/resource/781/57781)

8. ['Nonolib' library by Dr. Steven Simpson](http://www.lancaster.ac.uk/~simpsons/nonogram/howitworks)

9. [Rust and WebAssembly](https://rustwasm.github.io/docs/book/)
