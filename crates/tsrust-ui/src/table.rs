//! Intrinsic column constraints for the native table layout; text metrics come from shaped cells.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Column {
    pub min: f32,
    pub max: f32,
    pub constrained: bool,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Cell {
    pub span: usize,
    pub min: f32,
    pub max: f32,
    pub constrained: bool,
}

pub(crate) fn columns(rows: &[Vec<Cell>]) -> Result<Vec<Column>, String> {
    let mut count = 0;
    for row in rows {
        let mut width = 0usize;
        for cell in row {
            if !(1..=1000).contains(&cell.span) {
                return Err("Table colSpan must be an integer from 1 through 1000".into());
            }
            if !cell.min.is_finite() || !cell.max.is_finite() || cell.min < 0. || cell.max < cell.min {
                return Err("Table cell measurements must be finite, nonnegative and ordered".into());
            }
            width = width.checked_add(cell.span).ok_or("Table column count overflow")?;
        }
        count = count.max(width);
    }
    let mut columns = Vec::new();
    columns.try_reserve_exact(count).map_err(|_| "Table column allocation failed")?;
    columns.resize(count, Column::default());
    // Single columns establish preferences before spanning cells add their deficits.
    for spanning in [false, true] {
        for row in rows {
            let mut start = 0;
            for cell in row {
                let end = start + cell.span;
                if (cell.span > 1) == spanning {
                    let range = &mut columns[start..end];
                    let minimum = range.iter().map(|column| column.min).sum::<f32>();
                    let preferred = range.iter().map(|column| column.max).sum::<f32>();
                    let minimum_extra = (cell.min - minimum).max(0.);
                    let preferred_extra = (cell.max - preferred).max(0.);
                    for column in range {
                        column.constrained |= cell.constrained && cell.span==1;
                        let share = if preferred > 0. { column.max / preferred } else { 1. / cell.span as f32 };
                        column.min += minimum_extra * share;
                        column.max = (column.max + preferred_extra * share).max(column.min);
                    }
                }
                start = end;
            }
        }
    }
    totals(&columns)?;
    Ok(columns)
}

fn totals(columns: &[Column]) -> Result<(f32, f32), String> {
    if columns.iter().any(|column| !column.min.is_finite() || !column.max.is_finite() || column.min < 0. || column.max < column.min) {
        return Err("Table column measurements must be finite, nonnegative and ordered".into());
    }
    let min = columns.iter().map(|column| column.min).sum::<f32>();
    let max = columns.iter().map(|column| column.max).sum::<f32>();
    if !min.is_finite() || !max.is_finite() {
        return Err("Table intrinsic width exceeds the supported layout range".into());
    }
    Ok((min, max))
}

pub(crate) fn fit(columns: &[Column], available: f32) -> Result<Vec<f32>, String> {
    if !available.is_finite() || available < 0. {
        return Err("Table available width must be finite and nonnegative".into());
    }
    let (min, max) = totals(columns)?;
    if available <= min {
        return Ok(columns.iter().map(|column| column.min).collect());
    }
    if available < max {
        let fraction = (available - min) / (max - min);
        return Ok(columns.iter().map(|column| column.min + (column.max - column.min) * fraction).collect());
    }
    let extra = available - max;
    let unconstrained=columns.iter().any(|column|!column.constrained);
    let eligible=|column:&Column|!unconstrained||!column.constrained;
    let preferred=columns.iter().filter(|column|eligible(column)).map(|column|column.max).sum::<f32>();
    let count=columns.iter().filter(|column|eligible(column)).count();
    let equal = if columns.is_empty() { 0. } else { extra / count as f32 };
    Ok(columns.iter().map(|column| column.max + if !eligible(column){0.}else if preferred > 0. { extra * (column.max / preferred) } else { equal }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cell(span: usize, min: f32, max: f32) -> Cell { Cell { span, min, max, constrained:false } }

    #[test]
    fn headers_and_body_share_column_preferences_and_narrow_tables_overflow() {
        let columns = columns(&[vec![cell(1, 20., 40.), cell(1, 30., 100.)], vec![cell(1, 25., 60.), cell(1, 10., 80.)]]).unwrap();
        assert_eq!(columns, vec![Column { min: 25., max: 60., constrained:false }, Column { min: 30., max: 100., constrained:false }]);
        assert_eq!(fit(&columns, 20.).unwrap(), vec![25., 30.]);
        let intermediate = fit(&columns, 107.5).unwrap();
        assert_eq!(intermediate, vec![42.5, 65.]);
        assert_eq!(fit(&columns, 320.).unwrap(), vec![120., 200.]);
    }

    #[test]
    fn spans_constrain_shared_columns_without_changing_the_source_rows() {
        let rows = vec![vec![cell(1, 10., 20.), cell(1, 20., 80.)], vec![cell(2, 90., 160.)]];
        let columns = columns(&rows).unwrap();
        assert_eq!(columns, vec![Column { min: 22., max: 32., constrained:false }, Column { min: 68., max: 128., constrained:false }]);
        assert_eq!(rows[1][0].span, 2);
        assert_eq!(fit(&columns, 160.).unwrap(), vec![32., 128.]);
        assert!(super::columns(&[]).is_ok());
    }

    #[test]
    fn pixel_preferences_leave_surplus_for_unconstrained_columns(){
        let mut fixed=cell(1,100.,100.);fixed.constrained=true;
        let values=columns(&[vec![fixed,cell(1,40.,80.),cell(1,20.,40.)]]).unwrap();
        assert_eq!(fit(&values,400.).unwrap(),vec![100.,200.,100.]);
        assert_eq!(fit(&[Column{min:100.,max:100.,constrained:true}],200.).unwrap(),vec![200.]);
    }

    #[test]
    fn invalid_spans_and_nonfinite_geometry_fail_instead_of_clamping() {
        for value in [cell(0, 0., 0.), cell(1001, 0., 0.), cell(1, -1., 0.), cell(1, 2., 1.), cell(1, 0., f32::INFINITY)] {
            assert!(columns(&[vec![value]]).is_err());
        }
        assert!(fit(&[], f32::NAN).is_err());
        assert_eq!(fit(&[], 300.).unwrap(), Vec::<f32>::new());
    }
}
